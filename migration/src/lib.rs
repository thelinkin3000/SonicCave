pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20230120_000001_create_tables;
mod m20240124_122739_users;
mod m20240225_153942_users_extra_data;
mod m20240303_212729_playlists;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20230120_000001_create_tables::Migration),
            Box::new(m20240124_122739_users::Migration),
            Box::new(m20240225_153942_users_extra_data::Migration),
            Box::new(m20240303_212729_playlists::Migration),
        ]
    }
}
