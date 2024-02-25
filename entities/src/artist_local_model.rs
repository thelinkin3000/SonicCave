use sea_orm::DeriveIntoActiveModel;
use crate::artist::ActiveModel;

#[derive(DeriveIntoActiveModel, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ArtistModel {
    pub name: String,
    pub album_count: i32,
}
