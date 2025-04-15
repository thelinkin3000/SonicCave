use std::collections::HashMap;

use entities::album::Album;
use entities::artist::Artist;
use entities::song::Song;

use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub async fn handle_album(
    conn: &Pool<Postgres>,
    artist_id: Uuid,
    album_id_opt: Option<Uuid>,
    album: &Album,
    songs: &Vec<Song>,
) -> Result<(), sqlx::Error> {
    if let Some(album_id) = album_id_opt {
        let mut cloned = songs.clone();
        for s in &mut cloned {
            s.album_id = album_id;
        }
        let ret = queries::add_songs(conn, &cloned).await;
        ret?
    } else {
        let ret = queries::add_album(conn, Some(artist_id), album, songs).await;
        ret?
    }
    Ok(())
}

pub async fn sync_database(
    hashmap_to_add: HashMap<Artist, HashMap<Album, Vec<Song>>>,
    vec_to_delete: &Vec<String>,
    conn: &mut Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    let disk_artists: Vec<&Artist> = hashmap_to_add.keys().to_owned().collect();
    for disk_artist in disk_artists {
        let db_artist = queries::get_artist_by_name(conn, &disk_artist.name).await?;
        match db_artist {
            Some(artist) => {
                // Existing artist
                let albums = hashmap_to_add.get(disk_artist).unwrap();
                let db_albums = queries::get_albums_by_artist_id(conn, artist.id)
                    .await
                    .unwrap_or(Vec::new())
                    .into_iter();
                for (album, songs) in albums {
                    let existing_album_opt = db_albums.clone().find(|s| s.name == album.name);
                    let album_id = match existing_album_opt {
                        Some(a) => Some(a.id),
                        None => None,
                    };
                    handle_album(conn, artist.id, album_id, album, songs).await?;
                }
            }
            None => {
                // New Artist
                let artist_id_result = queries::add_artist(conn, disk_artist).await;
                let artist_id = artist_id_result?;
                let albums = hashmap_to_add.get(disk_artist).unwrap();
                for (album, songs) in albums {
                    queries::add_album(conn, Some(artist_id), album, songs).await?;
                }
            }
        }
    }
    let ret = queries::prune_songs(conn, vec_to_delete).await;
    match ret {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
