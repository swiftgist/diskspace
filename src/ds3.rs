use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;

struct DiskSpace {
    disk_space: BTreeMap<String, u64>,
}

impl DiskSpace {
    // fn update(&mut self, anchor: &String, file: PathBuf, file_size: u64,  matches: &ArgMatches)
    fn update(&mut self, anchor: &String, file: PathBuf, file_size: u64) {
        for ancestor in file.ancestors() {
            let ancestor_path = ancestor.to_string_lossy().to_string();
            let mut size: u64 = 0;
            if self.disk_space.contains_key(&ancestor_path) {
                size = *self.disk_space.get(&ancestor_path).unwrap();
            }
            size += file_size;
            self.disk_space.insert(ancestor_path.to_owned(), size);
            if anchor == &ancestor_path {
                break;
            }
            // if ancestor != file && matches.occurrences_of("parent") == 0 {
            //     break;
            // }
        }
    }

    fn visit(&mut self, dir: &String, paths: &Vec<PathBuf>) -> Vec<PathBuf> {
        // let mut pathnames = paths.to_vec();
        let pathnames = paths.to_vec();
        let dir_path = PathBuf::from(dir);

        if dir_path.is_dir() {
            for entry in fs::read_dir(dir_path).unwrap() {
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
                    self.visit(&path.to_string_lossy().to_string(), &paths);
                } else {
                    self.update(dir, path.to_owned(), path.metadata().unwrap().st_size());
                }
            }
        }
        pathnames
    }
}

pub fn traverse(anchor: &String, _matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut ds = DiskSpace {
        disk_space: BTreeMap::new(),
    };

    // let files = ds.visit(anchor, &Vec::<PathBuf>::new());
    let _files = ds.visit(anchor, &Vec::<PathBuf>::new());

    ds.disk_space
}
