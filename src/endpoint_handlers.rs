use std::collections::HashMap;

use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use entities::album_local_model::AlbumSqlxModel;
use entities::artist_local_model::ArtistSqlxModel;
use entities::song_local_model::SongSqlxModel;
use log::error;
use log::info;
use rand::seq::SliceRandom;
use rand::thread_rng;
use sea_orm::prelude::Uuid;
use sea_orm::QueryFilter;
use sea_orm::{
    ColIdx, ColumnTrait, DbBackend, EntityTrait, FromQueryResult, Order, QueryOrder, QuerySelect,
    Statement,
};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Execute, FromRow, Postgres, QueryBuilder, Row};

use entities::album::Column;
use entities::prelude::{Album, Artist};
use entities::{album, artist, song};

use crate::responses::album_response::AlbumResponse;
use crate::responses::responses::SearchResponse;
use crate::responses::responses::SearchResult;
use crate::responses::responses::{
    ArtistIndex, ArtistItem, ArtistResponse, ArtistsEndpointResponse, ArtistsEndpointResponseIndex,
    ErrorResponse, SubsonicResponse,
};
use crate::DatabaseState;

#[derive(Deserialize)]
pub struct GetAlbumsQuery {
    r#type: String,
    #[serde(default)]
    size: Option<i32>,
    #[serde(default)]
    offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct IdQuery {
    id: Uuid,
}

#[derive(Debug, FromQueryResult)]
struct IdOnly {
    id: Uuid,
}

#[derive(Deserialize, Clone)]
pub struct SearchQuery {
    query: String,
    artistCount: Option<i32>,
    artistOffset: Option<i32>,
    albumCount: Option<i32>,
    albumOffset: Option<i32>,
    songCount: Option<i32>,
    songOffset: Option<i32>,
}

#[derive(FromRow, Serialize)]
struct IdName {
    id: Uuid,
    name: String,
}

pub async fn search(
    State(state): State<DatabaseState>,
    query_option: Option<Query<SearchQuery>>,
) -> impl IntoResponse {
    if let None = query_option {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"required parameter "query" is missing"#.to_string(),
        );
        return Json(ret).into_response();
    }
    let query = query_option.unwrap().clone();

    let artist_rows = sqlx::query_as!(
        ArtistSqlxModel,
        r#"SELECT
	*
FROM "artist"
WHERE SIMILARITY(name,$1) > 0.4 or name ilike '%' || $1 || '%'
order by SIMILARITY(name,$1) desc
        LIMIT 10;"#,
        query.query
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let album_rows = sqlx::query_as!(
        AlbumSqlxModel,
        r#"select album.*, artist.name as artist_name
from album inner join artist on album.artist_id = artist.id
where SIMILARITY(album.name,$1) > 0.4 or album.name ilike '%' || $1 || '%'
or SIMILARITY(artist.name,$1) > 0.4 or artist.name ilike '%' || $1 || '%'
order by SIMILARITY(album.name,$1) + SIMILARITY(artist.name,$1) * 0.3 desc
        LIMIT 10;"#,
        query.query
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let song_rows = sqlx::query_as!(
        SongSqlxModel,
        r#"select song.*, album.name as album_name, artist.name as artist_name, album.year, artist.id as artist_id
        from song inner join album on song.album_id = album.id
                  inner join artist on album.artist_id = artist.id
        where SIMILARITY(song.title,$1) > 0.4 or song.title ilike '%' || $1 || '%'
            or SIMILARITY(album.name,$1) > 0.4 or album.name ilike '%' || $1 || '%'
        or SIMILARITY(artist.name,$1) > 0.4 or artist.name ilike '%' || $1 || '%'
        order by SIMILARITY(song.title,$1) + SIMILARITY(album.name,$1) * 0.3 + SIMILARITY(artist.name,$1) * 0.15 desc
        LIMIT 10;"#,
        query.query
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let ret =
        SubsonicResponse::<SearchResponse>::from_search_result(artist_rows, album_rows, song_rows);
    return Json(ret).into_response();
}

pub async fn get_album(
    State(state): State<DatabaseState>,
    query_option: Option<Query<IdQuery>>,
) -> impl IntoResponse {
    if let None = query_option {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"required parameter "id" is missing"#.to_string(),
        );
        return Json(ret).into_response();
    }

    let album_query = album::Entity::find()
        .filter(album::Column::Id.eq(query_option.unwrap().id))
        .one(&state.connection)
        .await;

    if let Err(err) = album_query {
        error!("Error fetching album: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let album_option = album_query.unwrap();

    if let None = album_option {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"resource with provided id does not exist"#.to_string(),
        );
        return Json(ret).into_response();
    }
    let album = album_option.unwrap();

    // Let's trust the referential integrity
    let artist = artist::Entity::find_by_id(album.artist_id)
        .one(&state.connection)
        .await
        .unwrap()
        .unwrap();
    let songs = song::Entity::find()
        .filter(song::Column::AlbumId.eq(album.id))
        .order_by(song::Column::DiscNumber, Order::Asc)
        .order_by(song::Column::Track, Order::Asc)
        .all(&state.connection)
        .await
        .unwrap();
    let ret = SubsonicResponse {
        subsonic_response: AlbumResponse::from_album(artist, album, songs),
    };
    Json(ret).into_response()
}

pub async fn get_albums(
    State(state): State<DatabaseState>,
    query_option: Option<Query<GetAlbumsQuery>>,
) -> impl IntoResponse {
    if let None = query_option {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"required parameter "type" is missing"#.to_string(),
        );
        return Json(ret).into_response();
    }
    let mut query = query_option.unwrap();
    if query.offset == None {
        query.offset = Some(0);
    }
    if query.size == None {
        query.size = Some(10)
    }
    let supported = vec![
        "frequent".to_string(),
        "newest".to_string(),
        "recent".to_string(),
        "random".to_string(),
        "alphabeticalByName".to_string(),
    ];
    if !supported.contains(&query.r#type) {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"required parameter "type" is missing"#.to_string(),
        );
        return Json(ret).into_response();
    }
    match query.r#type.as_str() {
        "random" => {
            let all_album_ids = IdOnly::find_by_statement(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "album"."id" FROM "album""#,
                vec![],
            ))
            .all(&state.connection)
            .await;
            if let Err(err) = all_album_ids {
                println!("52 {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            let mut album_ids: Vec<Uuid> =
                all_album_ids.unwrap().into_iter().map(|i| i.id).collect();
            album_ids.shuffle(&mut thread_rng());
            if query.size.unwrap() < album_ids.len() as i32 {
                album_ids = album_ids[0..query.size.unwrap() as usize].to_vec();
            }
            let albums_query = album::Entity::find()
                .filter(album::Column::Id.is_in(album_ids))
                .all(&state.connection)
                .await;
            if let Err(err) = albums_query {
                println!("62 {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            let albums: Vec<album::Model> = albums_query.unwrap();
            let artist_ids = albums
                .clone()
                .into_iter()
                .map(|i| i.artist_id)
                .collect::<Vec<Uuid>>();
            let artists_query = artist::Entity::find()
                .filter(artist::Column::Id.is_in(artist_ids))
                .all(&state.connection)
                .await;
            if let Err(err) = artists_query {
                println!("69 {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            let artists = artists_query.unwrap();
            let ret =
                SubsonicResponse::album_list2_from_album_list(albums.clone(), artists.clone());
            return Json(ret).into_response();
        }
        "frequent" | "newest" | "recent" | "alphabeticalByName" => {
            let albums_query = album::Entity::find()
                .order_by(album::Column::Name, Order::Asc)
                .offset(Some(query.offset.unwrap() as u64))
                .limit(Some(query.size.unwrap() as u64))
                .all(&state.connection)
                .await;
            if let Err(err) = albums_query {
                println!("62 {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            let albums: Vec<album::Model> = albums_query.unwrap();
            let artist_ids = albums
                .clone()
                .into_iter()
                .map(|i| i.artist_id)
                .collect::<Vec<Uuid>>();
            let artists_query = artist::Entity::find()
                .filter(artist::Column::Id.is_in(artist_ids))
                .all(&state.connection)
                .await;
            if let Err(err) = artists_query {
                println!("69 {}", err);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            let artists = artists_query.unwrap();
            let ret =
                SubsonicResponse::album_list2_from_album_list(albums.clone(), artists.clone());
            return Json(ret).into_response();
        }
        _ => {}
    }
    StatusCode::NO_CONTENT.into_response()
}

pub async fn get_artist(
    State(state): State<DatabaseState>,
    query_option: Option<Query<IdQuery>>,
) -> impl IntoResponse {
    if let None = query_option {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"required parameter "id" is missing"#.to_string(),
        );
        return Json(ret).into_response();
    }
    let query = query_option.unwrap();
    let artist_result = Artist::find()
        .filter(artist::Column::Id.eq(query.id))
        .one(&state.connection)
        .await;
    if let Err(err) = artist_result {
        error!("Error retrieving data from db: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let artist = artist_result.unwrap();
    if let None = artist {
        return StatusCode::NOT_FOUND.into_response();
    }
    let artist = artist.unwrap();
    let albums_result = Album::find()
        .filter(album::Column::ArtistId.eq(artist.id))
        .all(&state.connection)
        .await;
    if let Err(err) = albums_result {
        error!("Error retrieving data from db: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let albums = albums_result.unwrap();
    let ret = SubsonicResponse::artist_from_album_list(albums, artist);
    return Json(ret).into_response();
}

pub async fn get_artists(State(state): State<DatabaseState>) -> impl IntoResponse {
    let artists_result = Artist::find().all(&state.connection).await;
    if let Err(_) = artists_result {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let artists = artists_result.unwrap();
    let mut artists_hashmap: HashMap<String, Vec<ArtistItem>> = HashMap::new();
    for artist in artists {
        let name = artist
            .name
            .trim_start_matches("A ")
            .trim_start_matches("O ")
            .trim_start_matches("As ")
            .trim_start_matches("Os ")
            .trim_start_matches("Les ")
            .trim_start_matches("Le ")
            .trim_start_matches("Las ")
            .trim_start_matches("Los ")
            .trim_start_matches("La ")
            .trim_start_matches("El ")
            .trim_start_matches("The ");

        let first_letter;
        if name.len() > 0 {
            let char_index = name.char_indices().nth(1).unwrap_or((1, ' ')).0;
            first_letter = name[0..char_index].to_uppercase().clone();
        } else {
            first_letter = "".to_string();
        }

        if !artists_hashmap.contains_key(&first_letter) {
            artists_hashmap.insert(
                first_letter,
                vec![ArtistItem {
                    id: artist.id,
                    name: artist.name,
                    album_count: 0,
                    artist_image_url: "".to_string(),
                }],
            );
        } else {
            let mut vec: &mut Vec<ArtistItem> = artists_hashmap.get_mut(&first_letter).unwrap();
            vec.push(ArtistItem {
                id: artist.id,
                name: artist.name,
                album_count: 0,
                artist_image_url: "".to_string(),
            });
        }
    }
    let mut artists_endpoint_response: ArtistsEndpointResponse = ArtistsEndpointResponse {
        status: "ok".to_string(),
        version: "1.1.16".to_string(),
        r#type: "SonicCave".to_string(),
        server_version: "0.0.1".to_string(),
        artists: ArtistsEndpointResponseIndex { index: vec![] },
    };
    let mut keys: Vec<&String> = artists_hashmap.keys().into_iter().collect::<Vec<&String>>();
    keys.sort();
    for artist_key in keys {
        let mut artists_vec = artists_hashmap.get(artist_key).unwrap().to_vec();
        artists_vec.sort_by(|a, b| a.name.to_uppercase().cmp(&b.name.to_uppercase()));
        let index = ArtistIndex {
            name: artist_key.to_string(),
            artist: artists_vec,
        };
        artists_endpoint_response.artists.index.push(index);
    }
    let ret = SubsonicResponse {
        subsonic_response: artists_endpoint_response,
    };
    return Json(ret).into_response();
}
