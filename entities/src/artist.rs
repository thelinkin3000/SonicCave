//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.11

use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, FromRow, Hash)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub album_count: i32,
}
#[derive(FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct ArtistSqlxModel {
    pub album_count: i32,
    pub id: Uuid,
    pub name: String,
}
