use sea_orm::DeriveIntoActiveModel;
use crate::song::ActiveModel;

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
}