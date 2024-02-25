use chrono;
use chrono::Utc;
use sea_orm::prelude::Uuid;
use serde::Serialize;
use uuid;

use entities::{album, artist};

#[derive(Serialize, Clone)]
pub struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    pub(crate) subsonic_response: T,
}

#[derive(Serialize, Clone)]
pub struct ErrorResponse {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    #[serde(rename = "serverVersion")]
    pub(crate) server_version: String,
    pub(crate) error: ErrorResponseContainer,
}

#[derive(Serialize, Clone)]
pub struct ErrorResponseContainer {
    pub(crate) code: i32,
    pub(crate) message: String,
}

impl SubsonicResponse<ErrorResponse> {
    pub fn from_message(message: String) -> Self {
        Self {
            subsonic_response: {
                ErrorResponse {
                    status: "failed".to_string(),
                    version: "1.1.16".to_string(),
                    r#type: "soniccave".to_string(),
                    server_version: "0.0.1".to_string(),
                    error: ErrorResponseContainer { code: 0, message },
                }
            }
        }
    }
    pub fn from_error_code(code: i32, message: String) -> Self {
        Self {
            subsonic_response: {
                ErrorResponse {
                    status: "failed".to_string(),
                    version: "1.1.16".to_string(),
                    r#type: "soniccave".to_string(),
                    server_version: "0.0.1".to_string(),
                    error: ErrorResponseContainer { code, message },
                }
            }
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ArtistsEndpointResponse {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    #[serde(rename = "serverVersion")]
    pub(crate) server_version: String,
    pub(crate) artists: ArtistsEndpointResponseIndex,
}

#[derive(Serialize, Clone)]
pub struct ArtistsEndpointResponseIndex {
    pub(crate) index: Vec<ArtistIndex>,
}

#[derive(Serialize, Clone)]
pub struct ArtistIndex {
    pub(crate) name: String,
    pub(crate) artist: Vec<ArtistItem>,
}

#[derive(Serialize, Clone)]
pub struct ArtistItem {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    #[serde(rename = "albumCount")]
    pub(crate) album_count: i32,
    #[serde(rename = "artistImageUrl")]
    pub(crate) artist_image_url: String,
}

#[derive(Serialize, Clone)]
pub struct AlbumList2Response {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    #[serde(rename = "serverVersion")]
    pub(crate) server_version: String,
    #[serde(rename = "albumList2")]
    pub(crate) album_list2: AlbumList2,
}

#[derive(Serialize, Clone)]
pub struct AlbumList2 {
    pub(crate) album: Vec<AlbumList2Item>,
}

#[derive(Serialize, Clone)]
pub struct AlbumList2Item {
    pub(crate) id: Uuid,
    pub(crate) parent: Uuid,
    #[serde(rename = "isDir")]
    pub(crate) is_dir: bool,
    pub(crate) title: String,
    pub(crate) name: String,
    pub(crate) album: String,
    pub(crate) artist: String,
    pub(crate) year: i32,
    pub(crate) genre: String,
    #[serde(rename = "coverArt")]
    pub(crate) cover_art: Uuid,
    pub(crate) duration: i32,
    #[serde(rename = "playCount")]
    pub(crate) play_count: i32,
    pub(crate) created: chrono::DateTime<Utc>,
    #[serde(rename = "artistId")]
    pub(crate) artist_id: Uuid,
    #[serde(rename = "songCount")]
    pub(crate) song_count: i32,
    #[serde(rename = "isVideo")]
    pub(crate) is_video: bool,
}

impl SubsonicResponse<AlbumList2Response> {
    pub fn album_list2_from_album_list(list: Vec<album::Model>, artists_list: Vec<artist::Model>) -> Self {
        let mut ret = Vec::new();
        for item in list {
            // I'm sure I have the artist
            let artist = artists_list.iter().find(|i| i.id == item.artist_id).unwrap();
            ret.push(
                AlbumList2Item {
                    id: item.id,
                    parent: artist.id,
                    is_dir: true,
                    title: item.name.to_owned(),
                    name: item.name.to_owned(),
                    album: item.name.to_owned(),
                    artist: artist.name.to_owned(),
                    year: item.year,
                    genre: "".to_string(),
                    cover_art: Uuid::new_v4(),
                    duration: 0,
                    play_count: 0,
                    created: Utc::now(),
                    artist_id: artist.id,
                    song_count: item.song_count,
                    is_video: false,
                }
            )
        }
        Self {
            subsonic_response: AlbumList2Response {
                status: "ok".to_string(),
                version: "1.1.16".to_string(),
                r#type: "soniccave".to_string(),
                server_version: "0.0.1".to_string(),
                album_list2: AlbumList2 { album: ret },
            },
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ArtistResponse {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    #[serde(rename = "serverVersion")]
    pub(crate) server_version: String,
    pub(crate) artist: ArtistResponseItem,
}
#[derive(Serialize, Clone)]
pub struct ArtistResponseItem {
    id: Uuid,
    name: String,
    #[serde(rename = "albumCount")]
    album_count: i32,
    #[serde(rename = "artistImageUrl")]
    artist_image_url: String,
    album: Vec<AlbumList2Item>,
}

impl SubsonicResponse<ArtistResponse> {
    pub fn artist_from_album_list(list: Vec<album::Model>, artist: artist::Model) -> Self {
        let mut ret: Vec<AlbumList2Item> = list.iter().map(|item| {
            AlbumList2Item {
                id: item.id,
                parent: artist.id,
                is_dir: true,
                title: item.name.to_owned(),
                name: item.name.to_owned(),
                album: item.name.to_owned(),
                artist: artist.name.to_owned(),
                year: item.year,
                genre: "".to_string(),
                cover_art: Uuid::new_v4(),
                duration: 0,
                play_count: 0,
                created: Utc::now(),
                artist_id: artist.id,
                song_count: item.song_count,
                is_video: false,
            }
        }).collect();
        Self {
            subsonic_response: ArtistResponse {
                status: "ok".to_string(),
                version: "1.1.16".to_string(),
                r#type: "soniccave".to_string(),
                server_version: "0.0.1".to_string(),
                artist: ArtistResponseItem {
                    id: artist.id,
                    name: artist.name,
                    album_count: list.capacity() as i32,
                    artist_image_url: "".to_string(),
                    album: ret,
                },
            },
        }
    }
}