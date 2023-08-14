use clap::Parser;

use std::path::Path;
use std::fs;

use sha256::try_digest;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to organize
    #[arg(short, long)]
    dir: String,

    /// Exclude pattern
    #[arg(short, long, default_value="", required=false)]
    exclude: String,

    /// Include pattern
    #[arg(short, long, default_value="", required=false)]
    include: String,
}

fn organize(dir: &Path) -> i32 {
    if let Ok(dir_listing) = fs::read_dir(dir) {
        for entry in dir_listing {
            if let Ok(entry) = entry {
                // Here, `entry` is a `DirEntry`.
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        let subdir = entry.path();
                        organize(subdir.as_path());
                    } else {
                        if let Ok(digest) = try_digest(entry.path()) {
                            println!("{} {}", entry.path().display(), digest);
                        } else {
                            println!("IDK WTF happened there");
                        }
                    }
                } else {
                    println!("Couldn't get metadata for {:?}", entry.path());
                }
            }
        } 
    }
    0
}

fn main() {
    let args = Args::parse();

    println!("Searching {}!", args.dir);
    
    organize(Path::new(&args.dir));
}