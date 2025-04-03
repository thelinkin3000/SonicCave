use std::collections::HashMap;
use std::time::SystemTime;

use entities::album::Album;
use entities::artist::Artist;
use entities::song::Song;
use log::info;

use sqlx::{Pool, Postgres};
use uuid::Uuid;

async fn handle_artist(
    conn: &mut Pool<Postgres>,
    artist_id: Uuid,
    artist_model: &Artist,
    albums_hashmap: &HashMap<Album, Vec<Song>>,
) {
    // Handle all of its albums
    let mut disk_albums: Vec<&Album> = albums_hashmap.keys().into_iter().collect();
    let disk_albums_iter = disk_albums.clone().into_iter();
    let mut handled_albums: Vec<&Album> = Vec::new();
    let database_albums = queries::get_albums_by_artist_id(&conn, artist_model.id)
        .await
        .unwrap_or_else(|| Vec::new());
    for database_album in &database_albums {
        if let Some(album_model) = disk_albums_iter
            .clone()
            .find(|album_model| album_model.name.eq(&database_album.name))
        {
            // HANDLE SONGS
            let database_songs = queries::get_songs_by_album_id(&conn, album_model.id)
                .await
                .unwrap_or_else(|| Vec::new());
            let song_models: &mut Vec<Song> =
                &mut albums_hashmap.get(&album_model).unwrap().to_owned();
            for database_song in database_songs {
                if let Some(song_model) = song_models
                    .to_owned()
                    .into_iter()
                    .find(|song_model| song_model.title.eq(&database_song.title))
                {
                    // SONG EXISTS
                    song_models.remove(song_models.iter().position(|m| m.eq(&song_model)).unwrap());
                } else {
                    info!("Deleting song {}", database_song.id);
                    // Should delete it. It exists in database but not in filesystem
                    _ = queries::delete_song_by_id(&conn, database_song.id).await;
                }
            }

            if song_models.len() > 0 {
                // Let's add the songs!
                for song in &mut *song_models {
                    song.album_id = database_album.id;
                }
                _ = queries::add_songs(&conn, song_models).await;
            }

            handled_albums.push(album_model);
            println!("Album exists!");
        } else {
            println!("Album doesn't exist!");
            // Should delete it. It exists in database but not in filesystem
            _ = queries::delete_album_by_id(&conn, database_album.id).await;
        }
    }
    for album in handled_albums {
        disk_albums.retain(|s| s != &album);
    }

    for album in disk_albums {
        let album_songs = albums_hashmap.get(&album).unwrap();
        _ = queries::add_album(&conn, Some(artist_id), &album, album_songs).await;
    }
}

pub async fn sync_database(
    hashmap: HashMap<Artist, HashMap<Album, Vec<Song>>>,
    conn: &mut Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    let start = SystemTime::now();
    let mut disk_artists: Vec<&Artist> = hashmap.keys().to_owned().collect();
    let mut handled_artists_count = 0;
    let artists_page_size = 50;
    let mut artists_page = 0;
    while let Some(database_artists) =
        queries::search_artists_paginated(&conn, artists_page_size, artists_page).await
    {
        let mut handled_artists: Vec<&Artist> = Vec::new();
        info!("Fetched {artists_page_size} artits from the database");
        artists_page += 1;
        // Let's clone once and iterate a bunch of times
        let iter = disk_artists.clone().into_iter();
        for database_artist in database_artists {
            info!("Handling artist {}", database_artist.name);
            if let Some(artist_model) = iter
                .clone()
                .find(|artist_model| artist_model.name.eq(&database_artist.name))
            {
                // Artist exists in both. Syncing them.
                info!("Handling artist {}", database_artist.name);
                let albums_hashmap = hashmap.get(artist_model).unwrap();
                handle_artist(conn, database_artist.id, artist_model, albums_hashmap).await;
                handled_artists.push(&artist_model);
            } else {
                // Artist exists in database but not in filesystem. Deleting.
                _ = queries::delete_artist_by_id(&conn, database_artist.id).await;
            }
            handled_artists_count += 1;
        }
        // Mark filesystem artists as handled, so we don't process them again further down the line
        for handled_artist in handled_artists {
            disk_artists.retain(|s| s != &handled_artist)
        }
    }

    for artist in disk_artists {
        info!("Handling artist {}", artist.name);
        let artist_id = queries::add_artist(&conn, &artist).await?;
        for album_hashmap in hashmap.get(&artist).unwrap() {
            queries::add_album(&conn, Some(artist_id), album_hashmap.0, album_hashmap.1).await?;
        }
        handled_artists_count += 1;
    }
    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap().as_millis();
    println!(
        "Handled {} artists in {} ms",
        handled_artists_count, duration
    );
    Ok(())
}
