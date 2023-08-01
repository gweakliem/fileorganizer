use clap::Parser;

use std::path::Path;
use std::fs;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to organize
    #[arg(short, long)]
    dir: String,

    // / Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
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
                        println!("{}", entry.path().display());
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