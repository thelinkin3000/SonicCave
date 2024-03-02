use crate::song::ActiveModel;
use sea_orm::prelude::Uuid;
use sea_orm::DeriveIntoActiveModel;
use serde::Serialize;
use sqlx::FromRow;

#[derive(DeriveIntoActiveModel, PartialEq, Eq, Hash, Clone, Debug)]
pub struct SongModel {
    pub title: String,
    pub duration: i32,
    pub track: i32,
    pub album_id: sea_orm::prelude::Uuid,
    pub path: String,
    pub genre: String,
    pub suffix: String,
    pub content_type: String,
    pub disc_number: i32,
}

#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct SongSqlxModel {
    pub title: String,
    pub duration: i32,
    pub track: i32,
    pub album_id: sea_orm::prelude::Uuid,
    pub path: String,
    pub genre: String,
    pub suffix: String,
    pub content_type: String,
    pub disc_number: i32,
    pub id: Uuid,
    pub album_name: String,
    pub artist_name: String,
    pub year: i32,
    pub artist_id: Uuid,
}
