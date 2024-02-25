use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230120_000001_create_tables" // Make sure this matches with the file name
    }
}

#[derive(Iden)]
enum Artist {
    Table,
    Id,
    Name,
    AlbumCount,
}

#[derive(Iden)]
enum Album {
    Table,
    Id,
    Name,
    SongCount,
    Year,
    ArtistId,
}

#[derive(Iden)]
enum Song {
    Table,
    Id,
    Title,
    Track,
    Duration,
    AlbumId,
    Path,
    Genre,
    Suffix,
    ContentType
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create all three tables
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        println!("Running migration");
        // Create table for Artists
        manager
            .create_table(
                Table::create()
                    .table(Artist::Table)
                    .col(
                        ColumnDef::new(Artist::Id)
                            .uuid()
                            .not_null()
                            .extra("DEFAULT gen_random_uuid()")
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Artist::Name).string().not_null())
                    .col(ColumnDef::new(Artist::AlbumCount).integer().not_null())
                    .to_owned(),
            )
            .await?;

        // Create table for Albums
        manager.create_table(
            Table::create()
                .table(Album::Table)
                .col(ColumnDef::new(Album::Id).uuid().not_null().extra("DEFAULT gen_random_uuid()").primary_key())
                .col(ColumnDef::new(Album::Name).string().not_null())
                .col(ColumnDef::new(Album::Year).integer().not_null())
                .col(ColumnDef::new(Album::SongCount).integer().not_null())
                // Can't have an Album without an Artist
                .col(ColumnDef::new(Album::ArtistId).uuid().not_null())
                .foreign_key(ForeignKey::create().name("fk-album-artist_id").from(Album::Table, Album::ArtistId).to(Artist::Table, Artist::Id))
                .to_owned()
        )
            .await?;

        // Create table for Songs
        manager.create_table(
            Table::create()
                .table(Song::Table)
                .col(ColumnDef::new(Song::Id).uuid().extra("DEFAULT gen_random_uuid()").not_null().primary_key())
                .col(ColumnDef::new(Song::Title).string().not_null())
                .col(ColumnDef::new(Song::Path).string().not_null())
                .col(ColumnDef::new(Song::Genre).string().not_null())
                .col(ColumnDef::new(Song::Suffix).string().not_null())
                .col(ColumnDef::new(Song::ContentType).string().not_null())
                .col(ColumnDef::new(Song::Track).integer().not_null())
                .col(ColumnDef::new(Song::Duration).integer().not_null())
                // Can't have a song without an Album
                .col(ColumnDef::new(Song::AlbumId).uuid().not_null())
                .foreign_key(ForeignKey::create().name("fk-song-album_id").from(Song::Table, Song::AlbumId).to(Album::Table, Album::Id))
                .to_owned()
        )
            .await?;


        println!("Migration ran ok!");
        Ok(())
    }

    // Define how to roll back this migration: Drop all three tables
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Song::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Album::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Artist::Table).to_owned())
            .await?;
        Ok(())
    }
}