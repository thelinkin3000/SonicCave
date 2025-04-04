use async_recursion::async_recursion;
use id3::Tag;
use log::info;
use std::fs;

#[async_recursion]
pub async fn list(path: &str, indentation: usize, percentage: bool) -> Vec<String> {
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
        if is_dir {
            let inner = &mut list(&path, indentation + 2, false).await;
            info!("Parsing directory {}", &path);
            if !inner.is_empty() {
                ret.append(inner);
            }
        }
        // let tag_result: Option<SongTags> = tag();
        let tag_result = Tag::read_from_path(&path);
        match tag_result {
            Ok(_) => {
                ret.push(path);
            }
            Err(_) => {}
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
