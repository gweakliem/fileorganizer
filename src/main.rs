use clap::Parser;

use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

use sha256::try_digest;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to organize
    #[arg(short, long)]
    dir: String,

    /// Exclude pattern
    #[arg(short, long, default_value = "", required = false)]
    exclude: String,

    /// Include pattern
    #[arg(short, long, default_value = "", required = false)]
    include: String,
}

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
struct FileIndex {
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

    pub fn store_hash(&mut self, hash: String, file: File) -> () {
        let bucket = self.by_hash.entry(hash).or_insert(Vec::new());
        bucket.push(file);
    }

    pub fn get_hashes(&self) -> Iter<'_, String, Vec<File>> {
        self.by_hash.iter()
    }

    pub fn by_name(&self, name: &String) -> Option<&Vec<File>> {
        self.by_name.get(name)
    }

    pub fn store_name(&mut self, name: String, file: File) -> () {
        let path = Path::new(&name);
        // we set up a collision on duplicate file names
        let normalized = path.file_name().unwrap().to_string_lossy().to_string();
        // to fully implement, check to see if there's a prefix of this name already stored
        // then check if this name is a prefix of any existing name. Store this file under the shorter prefix
        // would require swapping keys sometimes.
        let bucket = self.by_name.entry(normalized).or_insert(Vec::new());
        bucket.push(file);
    }

    pub fn get_names(&self) -> Iter<'_, String, Vec<File>> {
        self.by_name.iter()
    }
}

fn walk_dir(dir: &Path, filter: fn(name: &str) -> bool) -> io::Result<Directory> {
    //println!("walk_dir {}", dir.to_str().unwrap());

    let entries: Vec<fs::DirEntry> = fs::read_dir(dir)?
        .filter_map(|result| result.ok())
        .collect();

    let mut directory: Vec<FileTree> = Vec::with_capacity(entries.len());

    for entry in entries {
        let path = entry.path();
        let name: String = path
            .file_name()
            .unwrap_or(OsStr::new("."))
            .to_str()
            .unwrap_or(".")
            .into();
        //println!("iter {}", name);
        if !filter(&name) {
            continue;
        };
        let metadata = fs::metadata(&path).unwrap();
        let node = match path {
            path if path.is_dir() => FileTree::DirNode(walk_dir(&path.as_path(), filter)?),
            path if path.is_symlink() => FileTree::LinkNode(Symlink {
                name: path.to_str().unwrap().into(),
                target: fs::read_link(path).unwrap().to_string_lossy().to_string(),
                metadata: metadata,
            }),
            path if path.is_file() => FileTree::FileNode(File {
                name: path.to_str().unwrap().into(),
                metadata: metadata,
            }),
            _ => unreachable!(),
        };
        directory.push(node);
    }
    let name = dir.to_str().unwrap().into();
    Ok(Directory {
        name: name,
        entries: directory,
    })
}

fn should_skip(file_name: &str) -> bool {
    return !file_name.starts_with(".");
}

fn visit_files<F>(node: &Directory, func: &mut F)
where
    F: FnMut(&File),
{
    for entry in &node.entries {
        match entry {
            FileTree::DirNode(sub_dir) => {
                visit_files(&sub_dir, func);
            }
            FileTree::LinkNode(_symlink) => {
                //let _digest = try_digest(Path::new(&symlink.name));
            }
            FileTree::FileNode(file) => {
                func(file);
            }
        }
    }
}

fn create_hash_index(node: &Directory, file_index: &mut FileIndex) -> () {
    let mut visitor = |file: &File| -> () {
        let digest = try_digest(Path::new(&file.name)).unwrap();
        file_index.store_hash(digest, file.clone());
    };
    visit_files(node, &mut visitor);
}

fn create_name_index(node: &Directory, file_index: & mut FileIndex) -> () {
    let mut visitor = |file: &File| -> () {
        let name = file.name.to_string();
        file_index.store_name(name, file.clone());
    };
    visit_files(node, &mut visitor);
}

fn organize(dir: &Path) -> i32 {
    let tree = walk_dir(dir, should_skip);
    match tree {
        Ok(tree) => {
            let mut file_index = FileIndex::new();
            create_hash_index(&tree, &mut file_index);
            for h in file_index.get_hashes() {
                let collisions = h.1;
                if collisions.len() > 1 {
                    println!("Multiple matches for {}", collisions.first().unwrap().name);
                    collisions.iter().for_each(|item| {
                        println!("\t {}", item.name);
                    });
                }
            }

            create_name_index(&tree, &mut file_index);

            for h in file_index.get_names() {
                let collisions = h.1;
                if collisions.len() > 1 {
                    println!("Multiple matches for {}", collisions.first().unwrap().name);
                    collisions.iter().for_each(|item| {
                        println!("\t {}", item.name);
                    });
                }
            }

            return 0;
        }
        Err(tree) => {
            println!("{}", tree);
            return 1;
        }
    }
}

fn main() {
    let args = Args::parse();

    println!("Searching {}!", args.dir);

    organize(Path::new(&args.dir));
}
