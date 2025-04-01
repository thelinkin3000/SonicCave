use std::collections::HashMap;
use std::time::SystemTime;

use log::info;
use sea_orm::prelude::Uuid;
use sea_orm::ColumnTrait;
use sea_orm::{
    ActiveValue, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};

use entities::album_local_model::AlbumModel;
use entities::artist_local_model::ArtistModel;
use entities::song_local_model::SongModel;
use entities::{album, artist, song};

pub async fn sync_database(
    hashmap: HashMap<ArtistModel, HashMap<AlbumModel, Vec<SongModel>>>,
    conn: &DatabaseConnection,
) -> Result<(), DbErr> {
    let connection = conn.to_owned();
    let start = SystemTime::now();

    let mut artist_pages = artist::Entity::find()
        .order_by_asc(artist::Column::Id)
        .paginate(&connection, 50);

    let mut artist_models: Vec<&ArtistModel> = hashmap.keys().to_owned().into_iter().collect();
    let mut artists_count = 0;
    while let Ok(Some(database_artists)) = artist_pages.fetch_and_next().await {
        for database_artist in database_artists {
            info!("Handling artist {}", database_artist.name);
            if let Some(artist_model) = artist_models
                .to_owned()
                .into_iter()
                .find(|artist_model| artist_model.name.eq(&database_artist.name))
            {
                // Handle all of its albums
                let albums_hashmap = hashmap.get(artist_model).unwrap();
                let mut album_models: Vec<&AlbumModel> = albums_hashmap.keys().collect();
                let database_albums = database_artist
                    .find_related(album::Entity)
                    .all(&connection)
                    .await?;
                for database_album in &database_albums {
                    info!("Handling album {}", database_album.name);
                    if let Some(album_model) = album_models
                        .to_owned()
                        .into_iter()
                        .find(|album_model| album_model.name.eq(&database_album.name))
                    {
                        // HANDLE SONGS
                        let database_songs = database_album
                            .find_related(song::Entity)
                            .all(&connection)
                            .await?;
                        let song_models: &mut Vec<SongModel> =
                            &mut albums_hashmap.get(&album_model).unwrap().to_owned();
                        for database_song in database_songs {
                            if let Some(song_model) = song_models
                                .to_owned()
                                .into_iter()
                                .find(|song_model| song_model.title.eq(&database_song.title))
                            {
                                // SONG EXISTS
                                song_models.remove(
                                    song_models.iter().position(|m| m.eq(&song_model)).unwrap(),
                                );
                            } else {
                                info!("Deleting song {}", database_song.id);
                                // Should delete it. It exists in database but not in filesystem
                                delete_song(database_song.id, &connection).await?;
                            }
                        }
                        if song_models.len() > 0 {
                            // Let's add the songs!
                            add_song(song_models, database_album.id, &connection).await?;
                        }
                        album_models.remove(
                            album_models
                                .iter()
                                .position(|m| m.eq(&album_model))
                                .unwrap(),
                        );
                        println!("Album exists!");
                    } else {
                        println!("Album doesn't exist!");
                        // Should delete it. It exists in database but not in filesystem
                        _ = delete_album(database_album.id, &connection).await;
                    }
                }
                for album in album_models {
                    let album_songs = albums_hashmap.get(&album).unwrap();
                    add_album(database_artist.id, album, album_songs, &connection).await?;
                }
                artist_models.remove(
                    artist_models
                        .iter()
                        .position(|m| m.eq(&artist_model))
                        .unwrap(),
                );
            } else {
                // Should delete it. It exists in database but not in filesystem.
                _ = delete_artist(database_artist.id, &connection.to_owned()).await;
            }
            println!("Handled {} artists", artists_count);
            artists_count += 1;
        }
    }

    for artist in artist_models {
        add_artist(&artist, hashmap.get(&artist).unwrap(), &connection).await?;
        println!("Handled {} artists", artists_count);
        artists_count += 1;
    }
    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap().as_millis();
    println!("Duration: {}ms", duration);
    return Ok(());
}

async fn delete_song(song_id: Uuid, connection: &DatabaseConnection) -> Result<(), DbErr> {
    let song_model = song::Entity::find_by_id(song_id)
        .one(*(&connection))
        .await?;
    let msg = format!("There isn't a song with id {}!", song_id);
    song_model.expect(&msg).delete(*(&connection)).await?;
    Ok(())
}

async fn delete_album(album_id: Uuid, connection: &DatabaseConnection) -> Result<(), DbErr> {
    let msg = format!("There isn't an album with id {}", album_id);
    let album = album::Entity::find_by_id(album_id)
        .one(*(&connection))
        .await?
        .expect(&msg);
    let songs = album.find_related(song::Entity).all(*(&connection)).await?;
    for song in songs {
        delete_song(song.id, &connection).await?;
    }
    album.delete(*(&connection)).await?;
    Ok(())
}

async fn delete_artist(artist_id: Uuid, connection: &DatabaseConnection) -> Result<(), DbErr> {
    let msg = format!("There isn't an artist with id {}", artist_id);
    let artist = artist::Entity::find_by_id(artist_id)
        .one(*(&connection))
        .await?
        .expect(&msg);
    let albums = album::Entity::find()
        .filter(album::Column::ArtistId.eq(artist.id))
        .all(*(&connection))
        .await?;
    for album in albums.to_owned() {
        delete_album(album.id, &connection).await?;
    }
    artist.delete(*(&connection)).await?;
    Ok(())
}

async fn add_album(
    artist_id: Uuid,
    album_model: &AlbumModel,
    songs: &Vec<SongModel>,
    connection: &DatabaseConnection,
) -> Result<(), DbErr> {
    let mut album_active_model = album_model.to_owned().into_active_model();
    album_active_model.artist_id = ActiveValue::Set(artist_id);
    let album_id = album::Entity::insert(album_active_model)
        .exec(*(&connection))
        .await?
        .last_insert_id;
    add_song(songs, album_id, &connection).await
}

async fn add_song(
    song_models: &Vec<SongModel>,
    album_id: Uuid,
    connection: &DatabaseConnection,
) -> Result<(), DbErr> {
    let mut song_vec: Vec<song::ActiveModel> = Vec::new().to_owned();
    song_models.iter().for_each(|song_model| {
        let mut song = song_model.to_owned().into_active_model();
        song.album_id = ActiveValue::Set(album_id);
        song_vec.push(song);
    });
    song::Entity::insert_many(song_vec.to_owned())
        .exec(*(&connection))
        .await?;

    Ok(())
}

async fn add_artist(
    artist_active_model: &ArtistModel,
    albums: &HashMap<AlbumModel, Vec<SongModel>>,
    connection: &DatabaseConnection,
) -> Result<(), DbErr> {
    let artist = (*artist_active_model).clone().into_active_model();
    let artist_id = artist::Entity::insert(artist)
        .exec(*(&connection))
        .await?
        .last_insert_id;
    let album_models = albums.keys();
    for album_active_model in album_models {
        add_album(
            artist_id,
            &album_active_model,
            &albums.get(album_active_model).unwrap(),
            &connection,
        )
        .await?;
    }
    Ok(())
}
