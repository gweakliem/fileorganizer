use clap::Parser;

use std::collections::hash_map::Entry;
use std::collections::hash_map::Iter;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Error;
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
    by_name: HashMap<String, File>,
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

fn visit(node: &Directory) -> Result<FileIndex, Error> {
    let mut file_index = FileIndex::new();

    for entry in &node.entries {
        match entry {
            FileTree::DirNode(sub_dir) => {
                println!("{}", sub_dir.name);
                visit(&sub_dir)?;
            }
            FileTree::LinkNode(symlink) => {
                let digest = try_digest(Path::new(&symlink.name));
                println!("{} -> {} {}", symlink.name, symlink.target, digest?);
                //fileIndex.store_hash(digest, file);
            }
            FileTree::FileNode(file) => {
                let digest = try_digest(Path::new(&file.name))?;
                //println!("{} {}", file.name, digest?);
                file_index.store_hash(digest, file.clone());
            }
        }
    }
    return Ok(file_index);
}

fn organize(dir: &Path) -> i32 {
    let tree = walk_dir(dir, should_skip);
    match tree {
        Ok(tree) => {
            let index = visit(&tree);
            //dbg!(index);
            match index {
                Ok(file_index) => {
                    for h in file_index.get_hashes() {
                        let collisions = h.1;
                        if collisions.len() > 0 {
                            println!("Multiple matches for {}", collisions.first().unwrap().name);
                        }
                    }
                }
                _ => {
                    println!("error reading index!");
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
