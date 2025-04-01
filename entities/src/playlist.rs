use serde::Serialize;
use sqlx::{
    types::{
        chrono::{DateTime, Local},
        Uuid,
    },
    FromRow,
};

#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub created: sea_orm::prelude::ChronoDateTime,
}

#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct PlaylistItem {
    pub id: Uuid,
    pub playlist_id: Uuid,
    pub song_id: Uuid,
    pub modified: DateTime<Local>,
}
