#![cfg(target_os = "linux")]
use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

/// Fifth implementation
/// I had the suspicion that parallel processing some parts of the
/// directory trees might help.  However, implementing threads in a
/// recursive function caused a bit of anguish.  I settled for only
/// creating a thread for directories of a certain st_size or greater.
/// In other words, keep directories with a few hundred files or less in
/// the main thread.
///
/// I abandoned this solution since experimenting saved one second on a
/// 200G home directory.  Additionally, this version can hit the number of
/// open files limit if the number of threads goes too high.

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

/// Visit_dirs
///
/// Recursive solution returning ().  Spawn threads for the largest
/// directories.
pub fn visit_dirs(dir: PathBuf, mds: &mut Mutex<BTreeMap<String, u64>>) {
    if dir.is_dir() {
        let mut children = vec![];
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
                if path.metadata().unwrap().st_size() > 500 {
                    let mut child_mds = Mutex::new(BTreeMap::new());
                    let _ = &children.push(thread::spawn(move || {
                        visit_dirs(path.to_owned(), &mut child_mds);
                        child_mds
                    }));
                } else {
                    visit_dirs(path.to_owned(), mds);
                }
            } else {
                let filesize = path.metadata().unwrap().st_size();
                for ancestor in path.ancestors() {
                    let ancestor_path = ancestor.to_string_lossy().to_string();
                    *mds.lock().unwrap().entry(ancestor_path).or_insert(0) += filesize;
                    if anchor == ancestor {
                        break;
                    }
                }
            }
        }
        for child in children {
            let subtree = child.join().unwrap();
            for (directory, size) in &*subtree.lock().unwrap() {
                mds.lock().unwrap().insert(directory.to_string(), *size);
            }
        }
    }
}
