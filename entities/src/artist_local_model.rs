use crate::artist::ActiveModel;
use sea_orm::{prelude::Uuid, DeriveIntoActiveModel};
use serde::Serialize;

#[derive(DeriveIntoActiveModel, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ArtistModel {
    pub name: String,
    pub album_count: i32,
}

#[derive(sqlx::FromRow, PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct ArtistSqlxModel {
    pub album_count: i32,
    pub id: Uuid,
    pub name: String,
}
