use std::path::PathBuf;

use crate::deepl;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn check_extension(ext: Option<&Vec<String>>, entry: &DirEntry) -> bool {
    log::trace!("ext : {:?}, entry : {:?}", ext, entry);
    if ext.is_none() {
        true
    } else {
        let ext = ext.unwrap().join("|");
        let ext = format!(".*\\.({})$", ext);
        let ext = Regex::new(&ext).unwrap();
        entry
            .file_name()
            .to_str()
            .map(|s| ext.is_match(s))
            .unwrap_or(false)
    }
}

pub fn new(deepl: &deepl::Deepl, path: PathBuf, max_depth: usize, hidden: bool) -> Vec<PathBuf> {
    log::trace!("start walkdir!!! path : {:?}", path);
    let walkdir = WalkDir::new(path).max_depth(max_depth).into_iter();
    log::trace!("walkdir : {:?}", walkdir);

    let walkdir = walkdir.filter_map(|e| e.ok());
    let target_name = deepl.config.project_name.as_str();
    let target_extensions = match &deepl.config.target_extensions {
        Some(ext_map) => {
            if ext_map.contains_key(target_name) {
                ext_map.get(target_name)
            } else {
                None
            }
        }
        None => None,
    };

    log::trace!("walkdir.size_hint().0 : {:?}", walkdir.size_hint().0);
    let mut walkdir_res = Vec::new();
    for e in walkdir {
        if (!hidden && is_hidden(&e)) || !check_extension(target_extensions, &e) {
            continue;
        }
        walkdir_res.push(e.into_path())
    }
    walkdir_res
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
