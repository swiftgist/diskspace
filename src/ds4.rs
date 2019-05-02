use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Mutex;

pub fn traverse(anchor: &String, _matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut mds = Mutex::new(BTreeMap::new());

    visit_dirs(PathBuf::from(anchor), &mut mds);

    let disk_space = mds.lock().ok().unwrap().clone();
    disk_space
}

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
    }
}
