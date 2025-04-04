use serde::Serialize;
use sqlx::{types::chrono::NaiveDateTime, FromRow};
use uuid::Uuid;

#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub created: NaiveDateTime,
}

#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct PlaylistItem {
    pub id: Uuid,
    pub playlist_id: Uuid,
    pub song_id: Uuid,
    pub modified: NaiveDateTime,
}
