use sea_orm::DeriveIntoActiveModel;
use crate::user::ActiveModel;

#[derive(DeriveIntoActiveModel, PartialEq, Eq, Hash, Clone, Debug)]
pub struct UserModel {
    pub username: String,
    pub password: String,
}