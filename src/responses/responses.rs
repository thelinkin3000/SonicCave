use serde::Serialize;
use sqlx::types::chrono;
use sqlx::types::chrono::{TimeZone, Utc};

#[derive(Serialize, Clone)]
pub struct subsonic_response<T> {
    #[serde(rename = "subsonic-response")]
    pub(crate) subsonic_response: T,
}

#[derive(Serialize, Clone)]
pub struct error_response {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    pub(crate) serverVersion: String,
    pub(crate) error: error_response_container,
}

#[derive(Serialize, Clone)]
pub struct error_response_container {
    pub(crate) code: i32,
    pub(crate) message: String,
}

impl error_response {
    fn from_message(message: String) -> Self {
        Self {
            status: "failed".to_string(),
            version: "1.1.16".to_string(),
            r#type: "soniccave".to_string(),
            serverVersion: "0.0.1".to_string(),
            error: error_response_container { code: 0, message },
        }
    }
}

#[derive(Serialize, Clone)]
pub struct artists_endpoint_response {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    pub(crate) serverVersion: String,
    pub(crate) artists: artists_endpoint_response_index,
}

#[derive(Serialize, Clone)]
pub struct artists_endpoint_response_index {
    pub(crate) index: Vec<artist_index>,
}

#[derive(Serialize, Clone)]
pub struct artist_index {
    pub(crate) name: String,
    pub(crate) artist: Vec<artist_response>,
}

#[derive(Serialize, Clone)]
pub struct artist_response {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) albumCount: i32,
    pub(crate) artistImageUrl: String,
}

#[derive(Serialize, Clone)]
pub struct album_list2_response {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    pub(crate) serverVersion: String,
    pub(crate) albumList2: album_list_2,
}

#[derive(Serialize, Clone)]
pub struct album_list_2 {
    pub(crate) album: Vec<album_list2_item>,
}

#[derive(Serialize, Clone)]
pub struct album_list2_item {
    pub(crate) id: i32,
    pub(crate) parent: i32,
    pub(crate) isDir: bool,
    pub(crate) title: String,
    pub(crate) name: String,
    pub(crate) album: String,
    pub(crate) artist: String,
    pub(crate) year: i32,
    pub(crate) genre: String,
    pub(crate) coverArt: String,
    pub(crate) duration: i32,
    pub(crate) playCount: i32,
    pub(crate) created: chrono::DateTime<Utc>,
    pub(crate) artistId: i32,
    pub(crate) songCount: i32,
    pub(crate) isVideo: bool,
}