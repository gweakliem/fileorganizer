mod file_tree;

use clap::Parser;
use file_tree::Directory;
use file_tree::File;
use file_tree::FileIndex;
use file_tree::FileTree;
use file_tree::Symlink;

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

fn walk_dir(dir: &Path, filter: fn(name: &str) -> bool) -> io::Result<Directory> {
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

// Could implement the --exclude filter here too
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

fn create_name_index(node: &Directory, file_index: &mut FileIndex) -> () {
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
                    println!(
                        "Probable duplicate content for {}",
                        collisions.first().unwrap().name
                    );
                    collisions.iter().for_each(|item| {
                        println!("\t {}", item.name);
                    });
                }
            }

            create_name_index(&tree, &mut file_index);

            for h in file_index.get_names() {
                let collisions = h.1;
                if collisions.len() > 1 {
                    println!(
                        "Possible filename duplicates for {}",
                        collisions.first().unwrap().name
                    );
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
