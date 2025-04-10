use async_recursion::async_recursion;
use log::{error, info};
use std::fs;

pub enum TagType {
    Id3,
    Flac,
}

#[async_recursion]
pub async fn list(
    path: &str,
    indentation: usize,
    percentage: bool,
    files_on_db: &mut Vec<String>,
) -> Vec<(String, TagType)> {
    let mut ret = Vec::new();
    let mut count = 0;
    if percentage {
        count = fs::read_dir(path).unwrap().count();
    }
    let paths = fs::read_dir(path).unwrap();
    let mut parsed = 0;
    for item in paths.map(|p| p.unwrap()) {
        let is_dir = item.file_type().unwrap().is_dir();
        let path: String = item.path().into_os_string().into_string().unwrap();
        let search_ret = files_on_db.binary_search(&path);
        if let Ok(index) = search_ret {
            files_on_db.remove(index);
            continue;
        }
        if is_dir {
            info!("Parsing directory {}", &path);
            let inner = &mut list(&path, indentation + 2, false, files_on_db).await;
            if !inner.is_empty() {
                ret.append(inner);
            }
            continue;
        }
        if path.ends_with(".flac") {
            if parse_flac(&path) {
                ret.push((path, TagType::Flac));
            } else if parse_id3(&path) {
                ret.push((path, TagType::Id3));
            } else {
                error!("File {path} does not have a tag we can read");
            }
        } else {
            if parse_id3(&path) {
                ret.push((path, TagType::Id3));
            } else if parse_flac(&path) {
                ret.push((path, TagType::Flac));
            } else {
                error!("File {path} does not have a tag we can read");
            }
        }
        if percentage {
            parsed += 1;
            info!(
                "Parsed {:.5}% of directories in parent",
                parsed as f32 * 100_f32 / count as f32
            );
        }
    }
    ret
}

pub fn parse_flac(path: &String) -> bool {
    let tag_result = metaflac::Tag::read_from_path(&path);
    match tag_result {
        Ok(_) => true,
        Err(_) => {
            error!("File {path} does not have flac tags we can read");
            false
        }
    }
}

pub fn parse_id3(path: &String) -> bool {
    let tag_result = id3::Tag::read_from_path(&path);
    match tag_result {
        Ok(_) => true,
        Err(_) => {
            error!("File {path} does not have id3 tags we can read");
            false
        }
    }
}
