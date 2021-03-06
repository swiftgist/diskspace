#![cfg(target_os = "linux")]
use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
#[cfg(test)]
use std::fs::File;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::Path;

/// Original implementation
/// The goal was to gain an understanding of Rust's Result and Option types
/// while learning some filesystem operations and structures.  I postponed
/// using recursion.
///

/// Traverse
///
/// Process each directory.  Skip symlinks.
pub fn traverse(anchor: &String, matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut disk_space: BTreeMap<String, u64> = BTreeMap::new();
    let mut directories: Vec<String> = Vec::new();

    directories.push(anchor.to_string());

    while directories.len() != 0 {
        let dir = directories.pop().unwrap();
        if fs::symlink_metadata(&dir).unwrap().file_type().is_symlink() {
            continue;
        }

        match fs::read_dir(&dir) {
            Ok(path) => process_path(anchor, path, &mut directories, &mut disk_space, &matches),
            Err(err) => println!("Cannot read directory {} - {}", dir, err),
        }
    }
    disk_space
}

/// Process Path
///
/// Find all directories and files in a path.  Append directories to a vector
/// and process each file
fn process_path(
    anchor: &String,
    path: fs::ReadDir,
    directories: &mut Vec<String>,
    disk_space: &mut BTreeMap<String, u64>,
    matches: &ArgMatches,
) {
    for entry in path.filter_map(|e| e.ok()) {
        match entry.path().to_str() {
            Some(pathname) => {
                if entry.path().is_dir() {
                    directories.push(pathname.to_string());
                } else {
                    process_file(anchor, entry, pathname.to_string(), disk_space, &matches)
                }
            }
            None => (),
        }
    }
}

/// Processs file
///
/// Add the file to the hash with its size and increment the parent by that size
fn process_file(
    top: &String,
    entry: fs::DirEntry,
    pathname: String,
    disk_space: &mut BTreeMap<String, u64>,
    matches: &ArgMatches,
) {
    match entry.metadata() {
        Ok(metadata) => {
            disk_space.insert(pathname, metadata.st_size());
            match entry.path().parent() {
                Some(parent) => match parent.to_str() {
                    Some(parent_pathname) => {
                        let path = Path::new(parent_pathname);
                        if matches.occurrences_of("parent") > 0 {
                            update_ancestors(top, path, &metadata, disk_space);
                        } else {
                            increment(parent_pathname, &metadata, disk_space);
                        }
                    }
                    None => (),
                },
                None => (),
            }
        }
        Err(err) => println!("Skipping {} - {}", pathname, err),
    }
}

/// Update Ancestors
///
/// Increment all parents of a file
fn update_ancestors(
    top: &String,
    path: &Path,
    metadata: &fs::Metadata,
    disk_space: &mut BTreeMap<String, u64>,
) {
    for ancestor in path.ancestors() {
        let ancestor_path = ancestor.to_str().unwrap();
        increment(ancestor_path, &metadata, disk_space);
        if top == ancestor_path {
            break;
        }
    }
}

/// Add the file size to the parent entry
///
/// Find the parent entry in the hash, record the current size and increment
/// by the st_size of the file.
fn increment(sparent: &str, metadata: &fs::Metadata, disk_space: &mut BTreeMap<String, u64>) {
    let mut size: u64 = 0;
    if disk_space.contains_key(sparent) {
        size = *disk_space.get(sparent).unwrap();
    }
    size += metadata.st_size();
    disk_space.insert(sparent.to_string(), size);
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        // No idea how to mock fs access yet
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(b"Hello World!").unwrap();
        let metadata = fs::metadata("foo.txt").unwrap();

        let parent = String::from("path/to/file");
        let mut data = BTreeMap::new();
        data.insert("path/to/file".to_string(), 0 as u64);

        increment(&parent, &metadata, &mut data);
        fs::remove_file("foo.txt");

        assert_eq!(data["path/to/file"], 12)
    }
}
