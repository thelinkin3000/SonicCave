use entities::{album::Album, artist::Artist, song::Song, user::User};
use log::error;
use sqlx::{Pool, Postgres, types::Uuid};
pub struct SongPath {
    path: String,
}

pub async fn get_song_paths(pool: &Pool<Postgres>) -> Result<Vec<String>, sqlx::Error> {
    let ret: Result<Vec<SongPath>, sqlx::Error> =
        sqlx::query_as!(SongPath, "select path from song;")
            .fetch_all(pool)
            .await;
    let res: Vec<String> = ret?.into_iter().map(|s| s.path).collect();
    Ok(res)
}

pub async fn search_artists_paginated(
    pool: &Pool<Postgres>,
    page_size: i64,
    page: i64,
) -> Option<Vec<Artist>> {
    let ret: Result<Vec<Artist>, sqlx::Error> = sqlx::query_as!(
        Artist,
        "select * from artist limit $1 offset $2",
        page_size,
        page * page_size
    )
    .fetch_all(pool)
    .await;
    if let Err(e) = ret {
        error!("There was an error querying the database: {}", e);
        return None;
    }
    let vec = ret.unwrap();
    if !vec.is_empty() { Some(vec) } else { None }
}

pub async fn get_albums_by_artist_id(pool: &Pool<Postgres>, artist_id: Uuid) -> Option<Vec<Album>> {
    let ret: Result<Vec<Album>, sqlx::Error> =
        sqlx::query_as!(Album, "select * from album where artist_id = $1", artist_id)
            .fetch_all(pool)
            .await;
    if let Err(e) = ret {
        error!("There was an error querying the database: {}", e);
        return None;
    }
    let vec = ret.unwrap();
    if !vec.is_empty() { Some(vec) } else { None }
}

pub async fn get_songs_by_album_id(pool: &Pool<Postgres>, album_id: Uuid) -> Option<Vec<Song>> {
    let ret: Result<Vec<Song>, sqlx::Error> = sqlx::query_as!(
        Song,
        "select * from song where album_id = $1 order by disc_number, track ",
        album_id
    )
    .fetch_all(pool)
    .await;
    if let Err(e) = ret {
        error!("There was an error querying the database: {}", e);
        return None;
    }
    let vec = ret.unwrap();
    if !vec.is_empty() { Some(vec) } else { None }
}

pub async fn delete_song_by_id(pool: &Pool<Postgres>, song_id: Uuid) -> Result<(), sqlx::Error> {
    let ret = sqlx::query!("delete from song where id = $1", song_id)
        .execute(pool)
        .await;
    ret?;
    Ok(())
}
pub async fn get_all_artists(pool: &Pool<Postgres>) -> Result<Vec<Artist>, sqlx::Error> {
    sqlx::query_as!(Artist, "select * from artist")
        .fetch_all(pool)
        .await
}
pub async fn get_user_by_username(
    pool: &Pool<Postgres>,
    username: &String,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"select * from "user" where username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_album_by_id(pool: &Pool<Postgres>, album_id: Uuid) -> Result<(), sqlx::Error> {
    let ret = sqlx::query!("delete from song where album_id = $1", album_id)
        .execute(pool)
        .await;
    ret?;

    let ret = sqlx::query!("delete from album where id = $1", album_id)
        .execute(pool)
        .await;
    ret?;
    Ok(())
}
pub async fn get_song_by_id(
    pool: &Pool<Postgres>,
    song_id: Uuid,
) -> Result<Option<Song>, sqlx::Error> {
    sqlx::query_as!(Song, "select * from song where id = $1", song_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_album_by_id(
    pool: &Pool<Postgres>,
    album_id: Uuid,
) -> Result<Option<Album>, sqlx::Error> {
    sqlx::query_as!(Album, "select * from album where id = $1", album_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_artist_by_id(
    pool: &Pool<Postgres>,
    artist_id: Uuid,
) -> Result<Option<Artist>, sqlx::Error> {
    sqlx::query_as!(Artist, "select * from artist where id = $1", artist_id)
        .fetch_optional(pool)
        .await
}
pub async fn get_artist_by_name(
    pool: &Pool<Postgres>,
    artist_name: &String,
) -> Result<Option<Artist>, sqlx::Error> {
    sqlx::query_as!(Artist, "select * from artist where name = $1", artist_name)
        .fetch_optional(pool)
        .await
}
pub async fn delete_artist_by_id(
    pool: &Pool<Postgres>,
    artist_id: Uuid,
) -> Result<(), sqlx::Error> {
    let albums = sqlx::query_as!(Album, "select * from album where artist_id = $1", artist_id)
        .fetch_all(pool)
        .await?;
    for album in albums {
        delete_album_by_id(pool, album.id).await?;
    }
    Ok(())
}
pub struct ReturnId {
    pub id: Uuid,
}
pub async fn prune_songs(pool: &Pool<Postgres>, paths: &Vec<String>) -> Result<(), sqlx::Error> {
    let mut ret = sqlx::query!(
        "delete from playlist_items where song_id in (select id from song where path = ANY($1))",
        paths
    )
    .execute(pool)
    .await;
    ret?;
    ret = sqlx::query!("delete from song where path =ANY($1)", paths)
        .execute(pool)
        .await;
    ret?;
    ret = sqlx::query!("delete from album where id not in (select distinct album_id from song)")
        .execute(pool)
        .await;
    ret?;
    ret = sqlx::query!("delete from artist where id not in (select distinct artist_id from album)")
        .execute(pool)
        .await;
    ret?;
    Ok(())
}
pub async fn add_artist(pool: &Pool<Postgres>, artist: &Artist) -> Result<Uuid, sqlx::Error> {
    let ret = sqlx::query_as! {
        ReturnId,
        "insert into artist (name, album_count) values ($1, $2) returning id",
        artist.name,
        artist.album_count
    }
    .fetch_one(pool)
    .await;
    Ok(ret?.id)
}
pub async fn add_album(
    pool: &Pool<Postgres>,
    artist_id: Option<Uuid>,
    album: &Album,
    songs: &Vec<Song>,
) -> Result<(), sqlx::Error> {
    let ret = sqlx::query_as!(
        ReturnId,
        r#"
        insert into album (name, year, song_count, artist_id)
        values ($1, $2, $3, $4)
        returning id
   "#,
        album.name,
        album.year,
        album.song_count,
        match artist_id {
            Some(id) => id,
            None => album.artist_id,
        },
    )
    .fetch_one(pool)
    .await;
    let album_id = ret?.id;
    let mut mut_songs = songs.to_owned();
    for song in &mut mut_songs {
        song.album_id = album_id;
    }
    let songs_ret = add_songs(pool, &mut_songs).await;
    songs_ret?;
    Ok(())
}

pub async fn add_songs(pool: &Pool<Postgres>, songs: &Vec<Song>) -> Result<(), sqlx::Error> {
    let mut title: Vec<String> = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut genre: Vec<String> = Vec::new();
    let mut suffix: Vec<String> = Vec::new();
    let mut content_type: Vec<String> = Vec::new();
    let mut track: Vec<i32> = Vec::new();
    let mut duration: Vec<i32> = Vec::new();
    let mut album_id: Vec<Uuid> = Vec::new();
    let mut disc_number: Vec<i32> = Vec::new();
    for song in songs {
        title.push(song.title.to_owned());
        path.push(song.path.to_owned());
        genre.push(song.genre.to_owned());
        suffix.push(song.suffix.to_owned());
        content_type.push(song.content_type.to_owned());
        track.push(song.track);
        duration.push(song.duration);
        album_id.push(song.album_id);
        disc_number.push(song.disc_number);
    }
    let ret = sqlx::query!(
        r#"
insert into song (title, path, genre, suffix, content_type, track, duration, album_id, disc_number) 
select * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::int[], $7::int[], $8::uuid[], $9::int[])            
        "#,
        &title[..],
        &path[..],
        &genre[..],
        &suffix[..],
        &content_type[..],
        &track[..],
        &duration[..],
        &album_id[..],
        &disc_number[..]
    ).execute(pool).await;
    ret?;
    Ok(())
}
