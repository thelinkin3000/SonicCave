use std::collections::HashMap;

use id3::{Tag, TagLike};

use entities::album;
use entities::artist;
use entities::song;

struct SongTags {
    artist: String,
    album: String,
    duration: i32,
    track: i32,
    year: i32,
    title: String,
    path: String,
}


pub fn parse(paths: Vec<String>) -> HashMap<artist::ArtistModel, HashMap<album::AlbumModel, Vec<song::SongModel>>> {
    let mut artists_map: HashMap<String, artist::ArtistModel> = HashMap::new();
    let mut artists_albums_map: HashMap<artist::ArtistModel, HashMap<album::AlbumModel, Vec<song::SongModel>>> = HashMap::new();
    println!("{}", paths.capacity());
    for item in paths {
        let tag_result: Option<SongTags> = tag(item.as_str());
        match tag_result {
            Some(song_tags) => {
                let artist_model: artist::ArtistModel;
                artist_model = artist::ArtistModel {
                    name: song_tags.artist.to_owned(),
                    album_count: 0,
                };
                // If we come across this artist for the first time we push it to the artists hashmap
                if (!artists_map.contains_key(&song_tags.artist)) {
                    artists_map.insert(song_tags.artist.to_owned(), artist_model.to_owned());
                    artists_albums_map.insert(artist_model.clone(), HashMap::new());
                }

                let album_model = album::AlbumModel {
                    name: song_tags.album.to_owned(),
                    year: song_tags.year.to_owned(),
                    artist_id: 0,
                    song_count: 0,
                };

                // If we don't already have this album, we add it
                if (!artists_albums_map.get(&artist_model).unwrap().contains_key(&album_model)) {
                    artists_albums_map.get_mut(&artist_model).unwrap().insert(album_model.to_owned(), Vec::new());
                }

                let song_model: song::SongModel = song::SongModel {
                    title: song_tags.title.to_owned(),
                    duration: song_tags.duration.to_owned(),
                    track: song_tags.track.to_owned(),
                    album_id: 0,
                    path: song_tags.path,
                };

                artists_albums_map.get_mut(&artist_model).unwrap().get_mut(&album_model.to_owned()).unwrap().push(song_model);
            }
            None => {}
        }
    }
    return artists_albums_map;
}

fn tag(path: &str) -> Option<SongTags> {
    let tag_result = Tag::read_from_path(path);
    let this_tag: Option<Tag>;
    match tag_result {
        Err(_) => return None,
        Ok(tag) => this_tag = Some(tag)
    }

    if let Some(tag) = this_tag {
        let artist = tag.album_artist().unwrap_or_else(|| "");
        let album = tag.album().unwrap_or_else(|| "");
        let title = tag.title().unwrap_or_else(|| "");
        let song = SongTags {
            artist: str::replace(artist, char::from(0), "?"),
            album: str::replace(album, char::from(0), "?"),
            duration: tag.duration().unwrap_or_else(|| 0) as i32,
            track: tag.track().unwrap_or_else(|| 0) as i32,
            year: tag.year().unwrap_or_else(|| 0) as i32,
            title: str::replace(title, char::from(0), "?"),
            path: path.to_string(),
        };
        return Some(song);
    }

    return None;
}
