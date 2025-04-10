use std::collections::HashMap;

use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::DateTime;
use chrono::Local;
use entities::playlist::Playlist;
use entities::song::SongSqlxModel;
use log::error;

use log::info;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::responses::album_response::AlbumResponse;
use crate::responses::responses::PlaylistResponse;
use crate::responses::responses::PlaylistsResponse;
use crate::responses::responses::SearchResponse;
use sqlx::postgres::PgQueryResult;
use sqlx::FromRow;

use crate::responses::responses::{
    ArtistIndex, ArtistItem, ArtistsEndpointResponse, ArtistsEndpointResponseIndex, ErrorResponse,
    SubsonicResponse,
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

#[derive(Deserialize, Serialize)]
pub struct IdQuery {
    id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct CountQuery {
    count: Option<i64>,
}

#[derive(Deserialize, Clone)]
pub struct SearchQuery {
    query: String,
    #[serde(rename = "artistCount")]
    artist_count: Option<i32>,
    #[serde(rename = "artistOffset")]
    artist_offset: Option<i32>,
    #[serde(rename = "albumCount")]
    album_count: Option<i32>,
    #[serde(rename = "albumOffset")]
    album_offset: Option<i32>,
    #[serde(rename = "songCount")]
    song_count: Option<i32>,
    #[serde(rename = "songOffset")]
    song_offset: Option<i32>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct CreatePlaylistQuery {
    name: Option<String>,
    #[serde(rename = "playlistId")]
    playlist_id: Option<Uuid>,
    #[serde(rename = "songId", default)]
    song_id: Option<Vec<Uuid>>,
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
        entities::artist::ArtistSqlxModel,
        r#"SELECT
	*
FROM "artist"
WHERE SIMILARITY(name,$1) > 0.4 or name ilike '%' || $1 || '%'
order by SIMILARITY(name,$1) desc
        LIMIT $2
        OFFSET $3;"#,
        query.query,
        query.artist_count.unwrap_or(10) as i64,
        query.artist_offset.unwrap_or(0) as i64
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let album_rows = sqlx::query_as!(
        entities::album::AlbumSqlxModel,
        r#"select album.*, artist.name as artist_name
        from album inner join artist on album.artist_id = artist.id
        where SIMILARITY(album.name,$1) > 0.4 or album.name ilike '%' || $1 || '%'
        or SIMILARITY(artist.name,$1) > 0.4 or artist.name ilike '%' || $1 || '%'
        order by SIMILARITY(album.name,$1) + SIMILARITY(artist.name,$1) * 0.3 desc
        LIMIT $2
        OFFSET $3;"#,
        query.query,
        query.album_count.unwrap_or(10) as i64,
        query.album_offset.unwrap_or(0) as i64
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let song_rows = sqlx::query_as!(
        entities::song::SongSqlxModel,
        r#"select song.id, song.title,song.path,song.genre,song.suffix,song.content_type,song.track,
        song.duration, song.album_id, song.disc_number,
         album.name as album_name, artist.name as artist_name, album.year, artist.id as artist_id
        from song inner join album on song.album_id = album.id
                  inner join artist on album.artist_id = artist.id
        where SIMILARITY(song.title,$1) > 0.4 or song.title ilike '%' || $1 || '%'
            or SIMILARITY(album.name,$1) > 0.4 or album.name ilike '%' || $1 || '%'
        or SIMILARITY(artist.name,$1) > 0.4 or artist.name ilike '%' || $1 || '%'
        order by SIMILARITY(song.title,$1) + SIMILARITY(album.name,$1) * 0.3 + SIMILARITY(artist.name,$1) * 0.15 desc
        LIMIT $2
        OFFSET $3;"#,
        query.query,
        query.song_count.unwrap_or(10) as i64,
        query.song_offset.unwrap_or(0) as i64
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let ret =
        SubsonicResponse::<SearchResponse>::from_search_result(artist_rows, album_rows, song_rows);
    Json(ret).into_response()
}

async fn get_db_playlist(
    State(state): State<DatabaseState>,
    id: Uuid,
) -> Result<Playlist, sqlx::Error> {
    sqlx::query_as!(Playlist, r#"select * from playlists where id = $1;"#, id)
        .fetch_one(&state.pool)
        .await
}

async fn get_db_songs_playlist(
    State(state): State<DatabaseState>,
    id: Uuid,
) -> Result<Vec<entities::song::SongSqlxModel>, sqlx::Error> {
    sqlx::query_as!(
        SongSqlxModel,
        r#"select song.id, song.title,song.path,song.genre,song.suffix,song.content_type,song.track,
        song.duration, song.album_id, song.disc_number,
         album.name as album_name, artist.name as artist_name, album.year, artist.id as artist_id
        from song inner join album on song.album_id = album.id
                  inner join artist on album.artist_id = artist.id
         where song.id in (
                select song_id from playlist_items where playlist_id = $1 
            );
        "#,
        id
    )
    .fetch_all(&state.pool)
    .await
}

pub async fn get_playlist(
    axum_state: State<DatabaseState>,
    id_query_option: Option<Query<IdQuery>>,
) -> impl IntoResponse {
    if let None = id_query_option {
        let ret: SubsonicResponse<ErrorResponse> = SubsonicResponse::from_error_code(
            10,
            r#"required parameter "id" is missing"#.to_string(),
        );
        return Json(ret).into_response();
    }
    let id_query = id_query_option.unwrap();
    let playlist_result = get_db_playlist(axum_state.to_owned(), id_query.id).await;
    match playlist_result {
        Err(err) => {
            error!("{}", err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        _ => (),
    }
    let songs_result = get_db_songs_playlist(axum_state.to_owned(), id_query.id).await;
    if let Err(err) = songs_result {
        error!("{}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Json(SubsonicResponse::<PlaylistResponse>::from_playlist(
        playlist_result.unwrap(),
        songs_result.unwrap(),
    ))
    .into_response()
}

pub async fn get_playlists(State(state): State<DatabaseState>) -> impl IntoResponse {
    let playlists_result = sqlx::query_as!(Playlist, r#"select * from playlists;"#)
        .fetch_all(&state.pool)
        .await;
    if let Err(err) = playlists_result {
        error!("{}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let playlists = playlists_result.unwrap();
    Json(SubsonicResponse::<PlaylistsResponse>::from_playlist_list(
        playlists,
    ))
    .into_response()
}

pub async fn create_update_playlist(
    axum_state: State<DatabaseState>,
    query_option: Option<axum_extra::extract::Query<CreatePlaylistQuery>>,
) -> impl IntoResponse {
    let State(state) = axum_state.to_owned();
    if let None = query_option {
        return StatusCode::NOT_FOUND.into_response();
    }
    let query = query_option.unwrap();
    let q: CreatePlaylistQuery = query.0;
    if q.name.is_none() && q.playlist_id.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let None = q.playlist_id {
        let ids = q.song_id.unwrap();
        let name = q.name.unwrap().to_owned();
        let playlist_insert_result = create_playlist(name, ids.to_owned(), &state.to_owned()).await;
        if let Err(err) = playlist_insert_result {
            error!("Error: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        let playlist_id = playlist_insert_result.unwrap();
        let playlist_result = get_db_playlist(axum_state.to_owned(), playlist_id).await;
        let songs_result = get_db_songs_playlist(axum_state, playlist_id).await;
        return Json(SubsonicResponse::<PlaylistResponse>::from_playlist(
            playlist_result.unwrap(),
            songs_result.unwrap(),
        ))
        .into_response();
    } else {
    }
    // info!("{}", serde_json::to_string(&q).unwrap());
    StatusCode::OK.into_response()
}

async fn update_playlist(
    State(state): State<DatabaseState>,
    id: Uuid,
    name: String,
    song_ids: Vec<Uuid>,
) -> Result<(), &'static str> {
    let playlist_result = sqlx::query_as!(
        CountQuery,
        r#"select count(*) from playlists where id = $1;"#,
        id
    )
    .fetch_one(&state.pool)
    .await;

    if let Err(err) = playlist_result {
        error!("{}", err);
        return Err("There was an error fetching the playlists.");
    }

    if playlist_result.unwrap().count.unwrap() != 1 {
        return Err("The provided id does not match any playlists in the database");
    }

    let songs_result = sqlx::query_as!(
        CountQuery,
        r#"select count(*) from song where id in (SELECT unnest($1::uuid[]));"#,
        song_ids.to_owned() as Vec<Uuid>
    )
    .fetch_one(&state.pool)
    .await;
    if let Err(err) = songs_result {
        error!("{}", err);
        return Err("There was an error fetching songs.");
    }

    let songs = songs_result.unwrap();
    let song_count = songs.count.unwrap();
    if song_count != song_ids.to_owned().into_iter().count() as i64 {
        return Err("At least one song id was not in the database");
    }
    let update_result = sqlx::query_as!(
        IdQuery,
        r#"
            update playlists set name = $1 where id = $2;
        "#,
        name,
        id
    )
    .fetch_one(&state.pool)
    .await;
    if let Err(err) = update_result {
        error!("{}", err);
        return Err("There was an error updating the playlist.");
    }
    info!("{}", song_count);
    let order: Vec<i32> = (0..song_count).map(|x| (x + 1) as i32).collect();
    let playlist_id_vec: Vec<Uuid> = (0..song_count).map(|_| id).collect();
    let modified_vec: Vec<DateTime<Local>> = (0..song_count).map(|_| Local::now()).collect();
    info!("{:?}", serde_json::to_string(&order.to_owned()));
    let remove_songs_result =
        sqlx::query!(r#"delete from playlist_items where playlist_id = $1"#, id)
            .execute(&state.pool)
            .await;
    if let Err(err) = remove_songs_result {
        error!("{}", err);
        return Err("There was an error deleting songs from the database.");
    }
    let insert_songs_result = insert_songs_query(
        playlist_id_vec,
        modified_vec,
        song_ids.to_owned(),
        order,
        &state,
    )
    .await;
    if let Err(err) = insert_songs_result {
        error!("{}", err);
        return Err("There was an error inserting songs in the playlist");
    }
    Ok(())
}

async fn insert_songs_query(
    playlist_id_vec: Vec<Uuid>,
    modified_vec: Vec<DateTime<Local>>,
    song_ids: Vec<Uuid>,
    order: Vec<i32>,
    state: &DatabaseState,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"insert into playlist_items (playlist_id, modified, song_id, item)
        SELECT * FROM UNNEST(
                        $1::UUID[],
                        $2::TIMESTAMP[],
                        $3::UUID[],
                        $4::INT4[]);
        "#,
        playlist_id_vec as Vec<Uuid>,
        modified_vec as Vec<DateTime<Local>>,
        song_ids.to_owned() as Vec<Uuid>,
        order as Vec<i32>
    )
    .execute(&state.pool)
    .await
}

async fn create_playlist(
    name: String,
    song_ids: Vec<Uuid>,
    state: &DatabaseState,
) -> Result<Uuid, &'static str> {
    let songs_result = sqlx::query_as!(
        CountQuery,
        r#"select count(*) from song where id in (SELECT unnest($1::uuid[]));"#,
        song_ids.to_owned() as Vec<Uuid>
    )
    .fetch_one(&state.pool)
    .await;
    if let Err(err) = songs_result {
        error!("{}", err);
        return Err("There was an error fetching songs.");
    }

    let songs = songs_result.unwrap();
    let song_count = songs.count.unwrap();
    if song_count != song_ids.to_owned().into_iter().count() as i64 {
        return Err("At least one song id was not in the database");
    }
    let insert_result = sqlx::query_as!(
        IdQuery,
        r#"
            INSERT INTO playlists (name, created)
            VALUES ($1, NOW())
            RETURNING id;
        "#,
        name
    )
    .fetch_one(&state.pool)
    .await;
    if let Err(err) = insert_result {
        error!("{}", err);
        return Err("There was an error fetching songs.");
    }
    let playlist_id = insert_result.unwrap().id;
    info!("{}", song_count);
    let order: Vec<i32> = (0..song_count).map(|x| (x + 1) as i32).collect();
    let playlist_id_vec: Vec<Uuid> = (0..song_count).map(|_| playlist_id).collect();
    let modified_vec: Vec<DateTime<Local>> = (0..song_count).map(|_| Local::now()).collect();
    info!("{:?}", serde_json::to_string(&order.to_owned()));

    let insert_songs_result = insert_songs_query(
        playlist_id_vec,
        modified_vec,
        song_ids.to_owned(),
        order,
        &state,
    )
    .await;
    if let Err(err) = insert_songs_result {
        error!("{}", err);
        return Err("There was an error inserting songs in the playlist");
    }
    Ok(playlist_id)
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

    let id = query_option.unwrap().id;

    let album_query = queries::get_album_by_id(&state.pool, id).await;

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
    let artist = queries::get_artist_by_id(&state.pool, album.artist_id)
        .await
        .unwrap()
        .unwrap();
    let songs = queries::get_songs_by_album_id(&state.pool, album.id)
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
        // "random" => {
        //     let all_album_ids = IdOnly::find_by_statement(Statement::from_sql_and_values(
        //         DbBackend::Postgres,
        //         r#"SELECT "album"."id" FROM "album""#,
        //         vec![],
        //     ))
        //     .all(&state.connection)
        //     .await;
        //     if let Err(err) = all_album_ids {
        //         println!("52 {}", err);
        //         return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //     }
        //     let mut album_ids: Vec<Uuid> =
        //         all_album_ids.unwrap().into_iter().map(|i| i.id).collect();
        //     album_ids.shuffle(&mut thread_rng());
        //     if query.size.unwrap() < album_ids.len() as i32 {
        //         album_ids = album_ids[0..query.size.unwrap() as usize].to_vec();
        //     }
        //     let albums_query = album::Entity::find()
        //         .filter(album::Column::Id.is_in(album_ids))
        //         .all(&state.connection)
        //         .await;
        //     if let Err(err) = albums_query {
        //         println!("62 {}", err);
        //         return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //     }
        //     let albums: Vec<album::Model> = albums_query.unwrap();
        //     let artist_ids = albums
        //         .clone()
        //         .into_iter()
        //         .map(|i| i.artist_id)
        //         .collect::<Vec<Uuid>>();
        //     let artists_query = artist::Entity::find()
        //         .filter(artist::Column::Id.is_in(artist_ids))
        //         .all(&state.connection)
        //         .await;
        //     if let Err(err) = artists_query {
        //         println!("69 {}", err);
        //         return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //     }
        //     let artists = artists_query.unwrap();
        //     let ret =
        //         SubsonicResponse::album_list2_from_album_list(albums.clone(), artists.clone());
        //     return Json(ret).into_response();
        // }
        // "frequent" | "newest" | "recent" | "alphabeticalByName" => {
        //     let albums_query = album::Entity::find()
        //         .order_by(album::Column::Name, Order::Asc)
        //         .offset(Some(query.offset.unwrap() as u64))
        //         .limit(Some(query.size.unwrap() as u64))
        //         .all(&state.connection)
        //         .await;
        //     if let Err(err) = albums_query {
        //         println!("62 {}", err);
        //         return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //     }
        //     let albums: Vec<album::Model> = albums_query.unwrap();
        //     let artist_ids = albums
        //         .clone()
        //         .into_iter()
        //         .map(|i| i.artist_id)
        //         .collect::<Vec<Uuid>>();
        //     let artists_query = artist::Entity::find()
        //         .filter(artist::Column::Id.is_in(artist_ids))
        //         .all(&state.connection)
        //         .await;
        //     if let Err(err) = artists_query {
        //         println!("69 {}", err);
        //         return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //     }
        //     let artists = artists_query.unwrap();
        //     let ret =
        //         SubsonicResponse::album_list2_from_album_list(albums.clone(), artists.clone());
        //     return Json(ret).into_response();
        // }
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
    let artist_result = queries::get_artist_by_id(&state.pool, query.id).await;
    if let Err(err) = artist_result {
        error!("Error retrieving data from db: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let artist = artist_result.unwrap();
    if let None = artist {
        return StatusCode::NOT_FOUND.into_response();
    }
    let artist = artist.unwrap();
    let albums_result = queries::get_albums_by_artist_id(&state.pool, artist.id).await;
    let albums = match albums_result {
        Some(a) => a,
        None => Vec::new(),
    };
    let ret = SubsonicResponse::artist_from_album_list(albums, artist);
    Json(ret).into_response()
}

pub async fn get_artists(State(state): State<DatabaseState>) -> impl IntoResponse {
    let artists_result = queries::get_all_artists(&state.pool).await;
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
            let vec: &mut Vec<ArtistItem> = artists_hashmap.get_mut(&first_letter).unwrap();
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
    Json(ret).into_response()
}
