use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE playlists
(
    id uuid default gen_random_uuid() not null primary key,
    name varchar not null,
    created timestamp not null
);

CREATE TABLE playlist_items
(
    id uuid default gen_random_uuid() not null primary key,
    playlist_id uuid not null,
    item int4 not null,
    song_id uuid not null,
    modified timestamp not null,
    CONSTRAINT fk_playlist_items_playlist
      FOREIGN KEY(playlist_id)
        REFERENCES playlists(id),
    CONSTRAINT fk_playlist_items_song
        FOREIGN KEY (song_id)
        REFERENCES song(id)
);"#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"DROP TABLE playlist_items;
DROP TABLE playlists;"#,
        )
        .await?;
        Ok(())
    }
}
