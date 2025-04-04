use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Hash)]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    pub year: i32,
    pub song_count: i32,
    pub artist_id: Uuid,
}

#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct AlbumSqlxModel {
    pub artist_id: Uuid,
    pub id: Uuid,
    pub name: String,
    pub song_count: i32,
    pub year: i32,
    pub artist_name: String,
}
