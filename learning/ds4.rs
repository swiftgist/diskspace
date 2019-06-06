use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Mutex;

/// Fourth implementation
/// For using recursion with simple functions, I found that Mutex is useful.
/// This version became the base for the implementation in ds.rs.  This is
/// the simplest so far.

/// Traverse
///
/// Creates a Mutex of a BTreeMap.  Locks and unwraps the result after
/// visiting all the files.
pub fn traverse(anchor: &String, _matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut mds = Mutex::new(BTreeMap::new());

    visit_dirs(PathBuf::from(anchor), &mut mds);

    let disk_space = mds.lock().ok().unwrap().clone();
    disk_space
}

/// Visit_Dirs
///
/// Recursive solution that returns ().  Using or_insert for default values
/// keeps the increment line concise.
pub fn visit_dirs(dir: PathBuf, mds: &mut Mutex<BTreeMap<String, u64>>) {
    if dir.is_dir() {
        let anchor = dir.to_owned();
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
                visit_dirs(path.to_owned(), mds);
            } else {
                let filesize = match path.metadata() {
                    #[cfg(target_os = "linux")]
                    Ok(metadata) => metadata.st_size(),
                    #[cfg(target_os = "windows")]
                    Ok(metadata) => metadata.file_size(),
                    Err(_) => 0,
                };
                for ancestor in path.ancestors() {
                    let ancestor_path = ancestor.to_string_lossy().to_string();
                    *mds.lock().unwrap().entry(ancestor_path).or_insert(0) += filesize;
                    if anchor == ancestor {
                        break;
                    }
                }
            }
        }
    }
}
