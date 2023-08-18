use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub metadata: fs::Metadata,
}

#[derive(Debug)]
pub struct Symlink {
    pub name: String,
    pub target: String,
    pub metadata: fs::Metadata,
}

#[derive(Debug)]
pub struct Directory {
    pub name: String,
    pub entries: Vec<FileTree>,
}

#[derive(Debug)]
pub enum FileTree {
    DirNode(Directory),
    FileNode(File),
    LinkNode(Symlink),
}

#[derive(Debug)]
pub struct FileIndex {
    by_hash: HashMap<String, Vec<File>>,
    by_name: HashMap<String, Vec<File>>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            by_hash: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn by_hash(&self, hash: &String) -> Option<&Vec<File>> {
        self.by_hash.get(hash)
    }

    pub fn store_hash(&mut self, hash: String, file: File) {
        let bucket = self.by_hash.entry(hash).or_insert(Vec::new());
        bucket.push(file);
    }

    pub fn get_hashes(&self) -> Iter<'_, String, Vec<File>> {
        self.by_hash.iter()
    }

    pub fn by_name(&self, name: &String) -> Option<&Vec<File>> {
        self.by_name.get(name)
    }

    pub fn store_name(&mut self, name: &String, file: File) {
        let path = Path::new(name);
        // we set up a collision on duplicate file names
        let normalized = path.file_stem().unwrap().to_string_lossy().to_string();
        let bucket = self.by_name.entry(normalized).or_insert(Vec::new());
        bucket.push(file);
    }

    pub fn get_names(&self) -> Iter<'_, String, Vec<File>> {
        self.by_name.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::file_tree::{File, FileIndex};

    #[test]
    fn identical_names_diff_path() {
        let mut file_index = FileIndex::new();

        let file1 = fs::File::open("test_files/file1.txt").unwrap();
        file_index.store_name(
            &"test_files/file1.txt".to_string(),
            File {
                name: "test_files/file1.txt".to_string(),
                metadata: file1.metadata().unwrap(),
            },
        );

        let file2 = fs::File::open("test_files/foo/file1.txt").unwrap();
        file_index.store_name(
            &"test_files/foo/file1.txt".to_string(),
            File {
                name: "test_files/foo/file1.txt".to_string(),
                metadata: file2.metadata().unwrap(),
            },
        );

        // Test that the files were stored correctly
        assert_eq!(
            file_index.by_name(&String::from("file1")).unwrap().len(),
            2
        );
    }

}
