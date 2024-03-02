use chrono;
use chrono::Utc;
use sea_orm::prelude::{DateTimeLocal, Uuid};
use serde::Serialize;

use entities::{
    album, album_local_model::AlbumSqlxModel, artist, artist_local_model::ArtistSqlxModel,
    song_local_model::SongSqlxModel,
};

use super::album_response::SongResponseData;

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
            },
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
            },
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
    pub fn album_list2_from_album_list(
        list: Vec<album::Model>,
        artists_list: Vec<artist::Model>,
    ) -> Self {
        let mut ret = Vec::new();
        for item in list {
            // I'm sure I have the artist
            let artist = artists_list
                .iter()
                .find(|i| i.id == item.artist_id)
                .unwrap();
            ret.push(AlbumList2Item {
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
            })
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
        let mut ret: Vec<AlbumList2Item> = list
            .iter()
            .map(|item| AlbumList2Item {
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
            })
            .collect();
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

#[derive(Serialize, Clone)]
pub struct SearchResponse {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    #[serde(rename = "serverVersion")]
    pub(crate) server_version: String,
    #[serde(rename = "searchResult3")]
    pub(crate) search_result3: SearchResult,
}

#[derive(Serialize, Clone)]
pub struct SearchResult {
    pub(crate) artist: Vec<ArtistItem>,
    pub(crate) album: Vec<AlbumList2Item>,
    pub(crate) song: Vec<SongResponseData>,
}

impl SubsonicResponse<SearchResponse> {
    pub fn from_search_result(
        artist_list: Vec<ArtistSqlxModel>,
        album_list: Vec<AlbumSqlxModel>,
        song_list: Vec<SongSqlxModel>,
    ) -> Self {
        let albums: Vec<AlbumList2Item> = album_list
            .iter()
            .map(|item| AlbumList2Item {
                id: item.id,
                parent: item.artist_id,
                is_dir: true,
                title: item.name.to_owned(),
                name: item.name.to_owned(),
                album: item.name.to_owned(),
                artist: item.artist_name.to_owned(),
                year: item.year,
                genre: "".to_string(),
                cover_art: Uuid::new_v4(),
                duration: 0,
                play_count: 0,
                created: Utc::now(),
                artist_id: item.artist_id,
                song_count: item.song_count,
                is_video: false,
            })
            .collect();

        let artists: Vec<ArtistItem> = artist_list
            .iter()
            .map(|item| ArtistItem {
                id: item.id,
                name: item.name.to_owned(),
                album_count: 0,
                artist_image_url: "".to_string(),
            })
            .collect();
        let songs: Vec<SongResponseData> = song_list
            .iter()
            .map(|item| SongResponseData {
                id: item.id,
                parent: item.album_id,
                is_dir: false,
                title: item.title.to_owned(),
                album: item.album_name.to_owned(),
                artist: item.artist_name.to_owned(),
                track: item.track,
                year: item.year,
                genre: item.genre.to_owned(),
                cover_art: "".to_string(),
                size: 0,
                content_type: item.content_type.to_owned(),
                suffix: item.suffix.to_owned(),
                duration: item.duration,
                bit_rate: 0,
                path: item.path.to_owned(),
                play_count: 0,
                disc_number: item.disc_number,
                created: Utc::now(),
                album_id: item.album_id,
                artist_id: item.artist_id,
                r#type: "audio".to_string(),
                is_video: false,
            })
            .collect();
        Self {
            subsonic_response: SearchResponse {
                status: "ok".to_string(),
                version: "1.1.16".to_string(),
                r#type: "soniccave".to_string(),
                server_version: "0.0.1".to_string(),
                search_result3: SearchResult {
                    artist: artists,
                    album: albums,
                    song: songs,
                },
            },
        }
    }
}
