use std::path::PathBuf;

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn check_extension(entry: &DirEntry) -> bool {
    // TODO: 呼び出し元のコマンドライン引数で、翻訳対象の拡張子を指定できるようにする
    let ext = Some(vec!["md"]);
    if ext.is_none() {
        true
    } else {
        let ext = ext.unwrap();
        let ext = ext.iter().map(|e| e.to_string()).collect::<Vec<String>>();
        let ext = ext.join("|");
        let ext = format!(".*\\.({})$", ext);
        let ext = Regex::new(&ext).unwrap();
        entry
            .file_name()
            .to_str()
            .map(|s| ext.is_match(s))
            .unwrap_or(false)
    }
}

pub fn new(
    path: PathBuf,
    max_depth: usize,
    // ext: Option<Vec<&str>>,
    hidden: bool,
) -> Vec<PathBuf> {
    println!("start walkdir!!! path : {:?}", path);
    let walkdir = WalkDir::new(path).max_depth(max_depth).into_iter();
    println!("walkdir : {:?}", walkdir);

    walkdir
        .filter_map(|e| e.ok())
        .map(|e| {
            if (!hidden && is_hidden(&e)) || !check_extension(&e) {
                return Err("");
            }
            Ok(e.into_path())
        })
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>()
}

// mod test {
//     use super::*;

//     #[test]
//     fn test_walkdir() {
//         let path = PathBuf::from(".");
//         let max_depth = 100;
//         let hidden = false;
//         let paths = new(path, max_depth, hidden);
//         println!("paths : {:?}", paths);
//     }
// }
