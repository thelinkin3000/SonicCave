use crate::album::ActiveModel;
use sea_orm::prelude::Uuid;
use sea_orm::DeriveIntoActiveModel;
use serde::Serialize;
use sqlx::FromRow;
#[derive(DeriveIntoActiveModel, PartialEq, Eq, Hash, Clone, Debug)]
pub struct AlbumModel {
    pub name: String,
    pub year: i32,
    pub artist_id: Uuid,
    pub song_count: i32,
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
