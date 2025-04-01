use async_recursion::async_recursion;
use id3::Tag;
use std::fs;

#[async_recursion]
pub async fn list(path: &str, indentation: usize) -> Vec<String> {
    let mut ret = Vec::new();
    let paths = fs::read_dir(path).unwrap();
    for item in paths.map(|p| p.unwrap()) {
        let is_dir = item.file_type().unwrap().is_dir();
        let path: String = item.path().into_os_string().into_string().unwrap();
        if is_dir {
            let inner = &mut list(&path, indentation + 2).await;
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
    }
    ret
}
