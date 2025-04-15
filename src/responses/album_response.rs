use chrono::{DateTime, Utc};
use entities::{album::Album, artist::Artist, song::Song};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct AlbumResponse {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) r#type: String,
    #[serde(rename = "serverVersion")]
    pub(crate) server_version: String,
    pub(crate) album: AlbumResponseData,
}

impl AlbumResponse {
    pub fn from_album(artist: Artist, album: Album, songs: Vec<Song>) -> Self {
        let mut duration = 0;
        let genre = songs[0].genre.to_owned();
        let songs_vec = songs
            .clone()
            .into_iter()
            .map(|i| {
                duration += i.duration;
                SongResponseData {
                    id: i.id,
                    parent: i.album_id,
                    is_dir: false,
                    title: i.title.to_string(),
                    album: album.name.to_string(),
                    artist: artist.name.to_string(),
                    track: i.track,
                    year: album.year,
                    genre: genre.to_string(),
                    cover_art: album.id.to_string(),
                    size: 0,
                    content_type: i.content_type,
                    suffix: i.suffix,
                    duration: i.duration,
                    bit_rate: 0,
                    path: i.path,
                    play_count: 0,
                    disc_number: 0,
                    created: Utc::now(),
                    album_id: album.id,
                    artist_id: artist.id,
                    r#type: "audio".to_string(),
                    is_video: false,
                }
            })
            .collect();
        let album_ret = AlbumResponseData {
            id: album.id,
            name: album.name,
            artist: artist.name,
            artist_id: artist.id,
            cover_art: "".to_string(),
            song_count: songs.to_owned().len() as i32,
            duration,
            play_count: 0,
            created: Utc::now(),
            year: album.year,
            genre,
            song: songs_vec,
        };
        Self {
            status: "ok".to_string(),
            version: "1.1.16".to_string(),
            r#type: "soniccave".to_string(),
            server_version: "0.0.1".to_string(),
            album: album_ret,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct AlbumResponseData {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) artist: String,
    #[serde(rename = "artistId")]
    pub(crate) artist_id: Uuid,
    #[serde(rename = "coverArt")]
    pub(crate) cover_art: String,
    #[serde(rename = "song_count")]
    pub(crate) song_count: i32,
    pub(crate) duration: i32,
    #[serde(rename = "playCount")]
    pub(crate) play_count: i32,
    pub(crate) created: DateTime<Utc>,
    pub(crate) year: i32,
    pub(crate) genre: String,
    pub(crate) song: Vec<SongResponseData>,
}

#[derive(Serialize, Clone)]
pub struct SongResponseData {
    pub(crate) id: Uuid,
    pub(crate) parent: Uuid,
    #[serde(rename = "isDir")]
    pub(crate) is_dir: bool,
    pub(crate) title: String,
    pub(crate) album: String,
    pub(crate) artist: String,
    pub(crate) track: i32,
    pub(crate) year: i32,
    pub(crate) genre: String,
    #[serde(rename = "coverArt")]
    pub(crate) cover_art: String,
    pub(crate) size: i64,
    #[serde(rename = "contentType")]
    pub(crate) content_type: String,
    pub(crate) suffix: String,
    pub(crate) duration: i32,
    #[serde(rename = "bitRate")]
    pub(crate) bit_rate: i32,
    pub(crate) path: String,
    #[serde(rename = "playCount")]
    pub(crate) play_count: i32,
    #[serde(rename = "discNumber")]
    pub(crate) disc_number: i32,
    pub(crate) created: DateTime<Utc>,
    #[serde(rename = "albumId")]
    pub(crate) album_id: Uuid,
    #[serde(rename = "artistId")]
    pub(crate) artist_id: Uuid,
    pub(crate) r#type: String,
    #[serde(rename = "isVideo")]
    pub(crate) is_video: bool,
}
