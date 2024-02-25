use sea_orm::DeriveIntoActiveModel;
use sea_orm::prelude::Uuid;
use crate::album::ActiveModel;

#[derive(DeriveIntoActiveModel, PartialEq, Eq, Hash, Clone, Debug)]
pub struct AlbumModel {
    pub name: String,
    pub year: i32,
    pub artist_id: Uuid,
    pub song_count: i32,
}