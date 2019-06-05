#![cfg(target_os = "linux")]
use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;

/// Second implementation
/// Attempted to simplify the number of functions using recursion.  However,
/// the double pass of building and then processing each file takes nearly
/// twice the time as the original implementation.

/// Discover
/// Gather all filenames and process each
pub fn traverse(anchor: &String, matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut disk_space: BTreeMap<String, u64> = BTreeMap::new();

    let files = visit_dirs(PathBuf::from(anchor), &Vec::<PathBuf>::new());

    for file in files {
        update_disk_space(
            anchor,
            file.to_path_buf(),
            file.metadata().unwrap().st_size(),
            &mut disk_space,
            matches,
        )
    }

    disk_space
}

/// Visit_dirs
/// Recursive function to walk a directory, skipping symlinks
pub fn visit_dirs(dir: PathBuf, paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut pathnames = paths.to_vec();

    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
            {
                continue;
            }
            if path.is_dir() {
                pathnames.push(path.to_owned());
                pathnames.extend(visit_dirs(path, &paths));
            } else {
                pathnames.push(path);
            }
        }
    }
    pathnames
}

/// Update_disk_space
/// Process file by adding size to its entry and each ancestor
pub fn update_disk_space(
    anchor: &String,
    file: PathBuf,
    file_size: u64,
    disk_space: &mut BTreeMap<String, u64>,
    matches: &ArgMatches,
) {
    for ancestor in file.ancestors() {
        let ancestor_path = ancestor.to_string_lossy().to_string();
        let mut size: u64 = 0;
        if disk_space.contains_key(&ancestor_path) {
            size = *disk_space.get(&ancestor_path).unwrap();
        }
        size += file_size;
        disk_space.insert(ancestor_path.to_owned(), size);
        if anchor == &ancestor_path {
            break;
        }
        if ancestor != file && matches.occurrences_of("parent") == 0 {
            break;
        }
    }
}
