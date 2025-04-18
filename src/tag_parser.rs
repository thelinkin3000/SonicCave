use std::collections::HashMap;
use std::fs::File;

use entities::album::Album;
use entities::artist::Artist;
use entities::song::Song;
use id3::{Tag, TagLike};
use log::error;
use symphonia::core::codecs::CODEC_TYPE_NULL;

use symphonia::core::formats::{FormatOptions, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use uuid::Uuid;

struct SongTags {
    artist: String,
    album: String,
    duration: i32,
    track: i32,
    year: i32,
    title: String,
    path: String,
    genre: String,
    suffix: String,
    content_type: String,
    disc_number: i32,
}

pub fn parse(
    paths: Vec<(String, crate::explorer::TagType)>,
) -> Result<HashMap<Artist, HashMap<Album, Vec<Song>>>, String> {
    let mut artists_map: HashMap<String, Artist> = HashMap::new();
    let mut artists_albums_map: HashMap<Artist, HashMap<Album, Vec<Song>>> = HashMap::new();
    println!("{}", paths.capacity());
    for item in paths {
        let tag_result: Option<SongTags> = tag(&item.0, item.1);
        if tag_result.is_none() {
            continue;
        }
        let song_tags = tag_result.unwrap();

        let artist_model: Artist = Artist {
            id: Uuid::nil(),
            name: song_tags.artist.to_owned(),
            album_count: 0,
        };
        // If we come across this artist for the first time we push it to the artists hashmap
        if !artists_map.contains_key(&song_tags.artist) {
            artists_map.insert(song_tags.artist.to_owned(), artist_model.to_owned());
            artists_albums_map.insert(artist_model.clone(), HashMap::new());
        }

        let album_model = Album {
            id: Uuid::nil(),
            name: song_tags.album.to_owned(),
            year: song_tags.year.to_owned(),
            artist_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            song_count: 0,
        };

        // If we don't already have this album, we add it
        if !artists_albums_map
            .get(&artist_model)
            .unwrap()
            .contains_key(&album_model)
        {
            artists_albums_map
                .get_mut(&artist_model)
                .unwrap()
                .insert(album_model.to_owned(), Vec::new());
        }
        let song_model: Song = Song {
            id: Uuid::nil(),
            title: song_tags.title.to_owned(),
            duration: song_tags.duration.to_owned(),
            track: song_tags.track.to_owned(),
            album_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            path: song_tags.path,
            genre: song_tags.genre,
            suffix: song_tags.suffix,
            content_type: song_tags.content_type,
            disc_number: song_tags.disc_number,
        };

        artists_albums_map
            .get_mut(&artist_model)
            .unwrap()
            .get_mut(&album_model.to_owned())
            .unwrap()
            .push(song_model);
    }
    Ok(artists_albums_map)
}

fn tag_id3(path: &str) -> Option<SongTags> {
    let tag_result = Tag::read_from_path(path);

    let this_tag: Option<Tag> = match tag_result {
        Err(_) => return None,
        Ok(tag) => Some(tag),
    };

    if let Some(tag) = this_tag {
        let path_split = path.split('.');
        let suffix = path_split
            .to_owned()
            .nth(path_split.into_iter().collect::<Vec<_>>().len() - 1)
            .unwrap_or("");
        let mut metadata_option = get_metadata(path.to_string(), suffix.to_string());
        if metadata_option.is_none() {
            // We have a tag but can't decode this.
            metadata_option = Some((Time::from(tag.duration().unwrap_or(0)), suffix.to_string()));
        }
        let metadata = metadata_option?;
        let artist = tag.album_artist().unwrap_or(tag.artist().unwrap_or(""));
        let album = tag.album().unwrap_or("");
        let title = tag.title().unwrap_or("");
        let genre = tag.genre().unwrap_or("");
        let song = SongTags {
            artist: str::replace(artist, char::from(0), "?"),
            album: str::replace(album, char::from(0), "?"),
            duration: metadata.0.seconds as i32,
            track: tag.track().unwrap_or(0) as i32,
            year: tag.year().unwrap_or(0),
            title: str::replace(title, char::from(0), "?"),
            path: path.to_string(),
            genre: str::replace(genre, char::from(0), "?"),
            suffix: suffix.to_string(),
            content_type: format!("audio/{}", metadata.1),
            disc_number: tag.disc().unwrap_or(1) as i32,
        };
        return Some(song);
    }
    None
}

fn parse_vorbis_comment(tag: &metaflac::Tag, tag_name: &str) -> Option<String> {
    let comment_vec_opt = tag.get_vorbis(tag_name);
    comment_vec_opt.as_ref()?;
    let mut comment_vec = comment_vec_opt.unwrap();
    let comment = comment_vec.next();
    comment?;
    Some(comment.unwrap().to_string())
}

fn parse_vorbis_comment_integer(tag: &metaflac::Tag, tag_name: &str) -> i32 {
    let track_str = parse_vorbis_comment(tag, tag_name).unwrap_or("".into());
    match track_str.as_str() {
        "" => 0,
        s => s.parse().unwrap_or(0),
    }
}
fn tag_flac(path: &str) -> Option<SongTags> {
    let tag_result = metaflac::Tag::read_from_path(path);

    let this_tag: Option<metaflac::Tag> = match tag_result {
        Err(_) => return None,
        Ok(tag) => Some(tag),
    };

    if let Some(tag) = this_tag {
        let path_split = path.split('.');
        let suffix = path_split.clone().last().unwrap_or("");
        let metadata_option = get_metadata(path.to_string(), suffix.to_string());
        let (duration, suffix): (i32, String) = match metadata_option {
            Some(metadata) => (metadata.0.seconds as i32, metadata.1.to_string()),
            None => (0, suffix.to_string()),
        };
        let artist = parse_vorbis_comment(&tag, "ALBUMARTIST").unwrap_or(
            parse_vorbis_comment(&tag, "ARTIST")
                .unwrap_or("".to_string())
                .to_string(),
        );
        let album = parse_vorbis_comment(&tag, "ALBUM").unwrap_or("".into());
        let title = parse_vorbis_comment(&tag, "TITLE").unwrap_or("".into());
        let genre = parse_vorbis_comment(&tag, "GENRE").unwrap_or("".into());
        let track = parse_vorbis_comment_integer(&tag, "TRACK");
        let year = parse_vorbis_comment_integer(&tag, "YEAR");
        let disc_number = parse_vorbis_comment_integer(&tag, "DISCNUMBER");
        let song = SongTags {
            artist: str::replace(&artist, char::from(0), "?"),
            album: str::replace(&album, char::from(0), "?"),
            duration,
            track,
            year,
            title: str::replace(&title, char::from(0), "?"),
            path: path.to_string(),
            genre: str::replace(&genre, char::from(0), "?"),
            suffix: suffix.to_string(),
            content_type: format!("audio/{}", suffix),
            disc_number,
        };
        return Some(song);
    }
    None
}
fn tag(path: &str, t: crate::explorer::TagType) -> Option<SongTags> {
    match t {
        crate::explorer::TagType::Id3 => tag_id3(path),
        crate::explorer::TagType::Flac => tag_flac(path),
    }
}

fn get_metadata(path: String, suffix: String) -> Option<(Time, String)> {
    // Open the media source.
    let src = File::open(&path).expect("failed to open media");

    // Create the media source stream.
    let media_source_stream = MediaSourceStream::new(Box::new(src), Default::default());

    // Create a probe hint using the file's extension. [Optional]
    let mut hint = Hint::new();
    hint.with_extension(suffix.as_str());
    // Use the default options for metadata and format readers.
    let metadata_opts: MetadataOptions = Default::default();
    let format_opts: FormatOptions = Default::default();

    // Probe the media source.
    let probed_result = symphonia::default::get_probe().format(
        &hint,
        media_source_stream,
        &format_opts,
        &metadata_opts,
    );
    if let Err(err) = probed_result {
        error!("Error parsing with Symphonia: {}", err);
        return None;
    }
    let probed = match probed_result {
        Ok(p) => p,
        Err(_) => return None,
    };
    let track_option = first_supported_track(probed.format.tracks());
    let track = track_option?;
    let params = &track.codec_params;
    let n_frames = params.n_frames?;
    let tb = params.time_base?;
    let time = tb.calc_time(n_frames);
    let codec = params.codec.to_string();
    Some((time, codec))
}

fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
    tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
}
