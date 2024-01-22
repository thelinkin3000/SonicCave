use std::fs;
use async_recursion::async_recursion;
use id3::{Tag, TagLike};

struct SongTags {
    artist: String,
    album: String,
    duration: i32,
    track: i32,
    year: i32,
    title: String,
}

#[async_recursion]
pub async fn list(
    path: &str,
    indentation: usize) -> Vec<String> {
    let mut ret = Vec::new();
    let paths = fs::read_dir(path).unwrap();
    for item in paths {
        let is_dir = item.as_ref().unwrap().file_type().unwrap().is_dir();
        if (is_dir) {
            let inner = &mut list(item.as_ref().unwrap().path().to_str().unwrap(), indentation + 2).await;
            if (!inner.is_empty()) {
                ret.append(inner);
            }
        }
        // let tag_result: Option<SongTags> = tag();
        let tag_result = Tag::read_from_path(item.as_ref().unwrap().path().to_str().unwrap());
        match tag_result {
            Ok(_) => {
                ret.push(item.as_ref().unwrap().path().to_str().unwrap().to_string());
            }
            Err(_) =>{}
        }
    }
    return ret;
}