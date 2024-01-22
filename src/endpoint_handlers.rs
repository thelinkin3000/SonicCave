use axum::extract::State;
use axum::response::IntoResponse;
use sea_orm::{ColIdx, EntityTrait};
use axum::extract::Query;
use entities::prelude::Artist;
use crate::DatabaseState;
use std::collections::HashMap;
use axum::http::StatusCode;
use axum::Json;
use crate::responses::responses::{artist_index, artist_response, artists_endpoint_response, artists_endpoint_response_index, subsonic_response};


struct GetAlbumQuery {


}

pub async fn get_albums(State(state): State<DatabaseState>, query: Option<Query<GetAlbumQuery>>) -> impl IntoResponse {
    if let None = query {

    }
    let artists_result = Artist::find();
}

pub async fn get_artists(State(state): State<DatabaseState>) -> impl IntoResponse {
    let artists_result = Artist::find().all(&state.connection).await;
    if let Err(_) = artists_result {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let artists = artists_result.unwrap();
    let mut artists_hashmap: HashMap<String, Vec<artist_response>> = HashMap::new();
    for artist in artists {
        let name = artist.name
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
        if (name.len() > 0) {
            let char_index = name.char_indices().nth(1).unwrap_or((1, ' ')).0;
            first_letter = name[0..char_index].to_uppercase().clone();
        } else {
            first_letter = "".to_string();
        }

        if (!artists_hashmap.contains_key(&first_letter)) {
            artists_hashmap.insert(first_letter, vec![artist_response {
                id: artist.id,
                name: artist.name,
                albumCount: 0,
                artistImageUrl: "".to_string(),
            }]);
        } else {
            let mut vec: &mut Vec<artist_response> = artists_hashmap.get_mut(&first_letter).unwrap();
            vec.push(
                artist_response {
                    id: artist.id,
                    name: artist.name,
                    albumCount: 0,
                    artistImageUrl: "".to_string(),
                });
        }
    }
    let mut artists_endpoint_response: artists_endpoint_response = artists_endpoint_response {
        status: "ok".to_string(),
        version: "1.1.16".to_string(),
        r#type: "SonicCave".to_string(),
        serverVersion: "0.0.1".to_string(),
        artists: artists_endpoint_response_index {
            index: vec![],
        },
    };
    let mut keys: Vec<&String> = artists_hashmap.keys().into_iter().collect::<Vec<&String>>();
    keys.sort();
    for artist_key in keys {
        let mut artists_vec = artists_hashmap.get(artist_key).unwrap().to_vec();
        artists_vec.sort_by(|a, b| {
            a.name.to_uppercase().cmp(&b.name.to_uppercase())
        });
        let index = artist_index {
            name: artist_key.to_string(),
            artist: artists_vec,
        };
        artists_endpoint_response.artists.index.push(index);
    }
    let ret = subsonic_response {
        subsonic_response: artists_endpoint_response,
    };
    return Json(ret).into_response();
}