//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.11

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "song")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub path: String,
    pub genre: String,
    pub suffix: String,
    pub content_type: String,
    pub track: i32,
    pub duration: i32,
    pub album_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::album::Entity",
        from = "Column::AlbumId",
        to = "super::album::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Album,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Album.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
