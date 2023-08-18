# File Organizer

I wanted to learn some Rust, and I have a terabyte drive full of old pictures that are super disorganized across many directories. My idea is to try to determine which of these pictures are duplicates and attempt to organize them.

Detect duplicates
* same image different format
* same image, resized or rotated
* similar images

Duplicate detection by 
* File Name
* SHA-256 hash
* EXIF Metadata
* Image similarity (resized, rotated, similar image)

Future Directions
path! macro
https://docs.rs/path_macro/latest/path_macro/

Image comparisons
https://crates.io/crates/image-compare
https://github.com/bokuweb/lcs-image-diff-rs
https://github.com/kornelski/dssim