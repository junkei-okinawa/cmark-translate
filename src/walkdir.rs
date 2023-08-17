use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

pub struct WalkDir {
    root: Box<dyn Iterator<Item = io::Result<DirEntry>>>,
    children: Box<dyn Iterator<Item = WalkDir>>,
}

impl WalkDir {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let root = Box::new(fs::read_dir(&path)?);
        let children = Box::new(fs::read_dir(&path)?.filter_map(|e| {
            let e = e.ok()?;
            if e.file_type().ok()?.is_dir() {
                return Some(WalkDir::new(e.path()).ok()?);
            }
            None
        }));
        Ok(WalkDir { root, children })
    }

    pub fn entries(self) -> Box<dyn Iterator<Item = io::Result<DirEntry>>> {
        Box::new(
            self.root
                .chain(self.children.map(|s| s.entries()).flatten()),
        )
    }
}

impl Iterator for WalkDir {
    type Item = io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.root.next() {
            return Some(item);
        }
        if let Some(child) = self.children.next() {
            self.root = child.entries();
            return self.next();
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn _test_visit_dir() {
        let paths = WalkDir::new("./src")
            .unwrap()
            .filter_map(|e| Some(e.ok()?.path()))
            .collect::<Vec<_>>();
        let test_path = std::path::PathBuf::new().join("./src/lib.rs");
        let mut test_paths = Vec::new();
        test_paths.push(test_path);

        assert_eq!(&paths, &test_paths);
    }
}
