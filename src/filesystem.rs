use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use swc_common::FileLoader;

pub struct InMemFileSystem {
    files: HashMap<String, String>,
}

impl InMemFileSystem {
    pub fn new() -> Self {
        InMemFileSystem {
            files: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn from(files: HashMap<String, String>) -> Self {
        InMemFileSystem { files }
    }

    pub fn save(&mut self, filename: &str, content: &str) {
        self.files.insert(filename.to_owned(), content.to_owned());
    }

    pub fn get(&self, filename: &str) -> Option<String> {
        self.files.get(filename).map(|content| content.to_owned())
    }

    pub fn exists(&self, filename: &str) -> bool {
        self.files.contains_key(filename)
    }
}

impl fmt::Debug for InMemFileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemFileSystem")
            .field("files", &self.files)
            .finish()
    }
}

#[cfg(test)]
mod in_mem_file_system_tests {
    use super::*;

    #[test]
    fn it_saves_a_file() {
        let mut fs = InMemFileSystem::new();

        fs.save("filename", "content");

        assert_eq!(
            HashMap::from([(String::from("filename"), String::from("content"))]),
            fs.files
        );
    }

    #[test]
    fn it_overwrites_a_file() {
        let mut fs = InMemFileSystem::from(HashMap::from([(
            String::from("filename"),
            String::from("content"),
        )]));

        fs.save("filename", "new content");

        assert_eq!(
            HashMap::from([(String::from("filename"), String::from("new content"))]),
            fs.files
        );
    }

    #[test]
    fn it_gets_a_file() {
        let fs = InMemFileSystem::from(HashMap::from([(
            String::from("filename"),
            String::from("content"),
        )]));

        let content = fs.get("filename");

        match content {
            Some(content) => assert_eq!("content", content),
            None => panic!("expect file to exist"),
        }
    }
}

pub struct InMemFileLoader {
    fs: Arc<Mutex<InMemFileSystem>>,
}

impl InMemFileLoader {
    pub fn new(fs: Arc<Mutex<InMemFileSystem>>) -> InMemFileLoader {
        InMemFileLoader { fs }
    }
}

impl FileLoader for InMemFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        self.fs.lock().unwrap().exists(&path.to_string_lossy())
    }

    fn abs_path(&self, path: &Path) -> Option<PathBuf> {
        if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            Some(PathBuf::from("/"))
        }
    }

    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        use std::io::{Error, ErrorKind};
        let fs = self.fs.lock().unwrap();
        let path = path.to_string_lossy();

        match fs.get(&path) {
            Some(value) => Ok(value),
            None => Err(Error::new(
                ErrorKind::NotFound,
                format!("{} does not exist", path),
            )),
        }
    }
}
