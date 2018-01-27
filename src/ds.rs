
pub mod diskspace {
    use std::iter::FromIterator;
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::os::linux::fs::MetadataExt;

    pub fn traverse(mut directories: Vec<String>) -> BTreeMap<String, u64> {
        let mut disk_space: BTreeMap<String, u64> = BTreeMap::new();

        while directories.len() != 0 {
            let dir = directories.pop().unwrap();
            // println!("{:?}", dir);
            let path = Path::new(&dir).read_dir();
            if let Ok(path) = path {
                for entry in path {
                    if let Ok(entry) = entry {
                        let epathname = entry.path();
                        let rpathname = epathname.to_str();
                        if let Some(rpathname) = rpathname {
                            let pathname = rpathname.to_string();
                            if entry.path().is_dir() {
                                directories.push(pathname);
                            } else {
                                let metadata = entry.metadata().expect("metadata call failed");
                                disk_space.insert(pathname, metadata.st_size());
                                let parent = entry.path().parent().unwrap().to_str().unwrap().to_string();
                                let mut size: u64 = 0;
                                if disk_space.contains_key(&parent) {
                                    size = *disk_space.get(&parent).unwrap();
                                }
                                size += metadata.st_size();
                                disk_space.insert(parent, size);
                            }
                        }
                    }
                }
            } else {
                println!("Problem with  {}", dir);
            }
        }
        disk_space

    }

    pub fn report(disk_space: BTreeMap<String, u64>) {

        let mut sorted = Vec::from_iter(disk_space);
        sorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

        let mut index = 0;
        for (filename, size) in sorted {
            if index > 19 { 
                break;
            }
            println!("{} {}", simple_units(size), filename);
            index += 1;
        }

    }

    pub fn simple_units(number: u64) -> String {
        let units = [ " ", "K", "M", "G", "T", "P" ];
        let mut index = 0;
        let mut n = number;
        while n > 1024 {
            index += 1;
            n = n/1024;
        }
        if index == 0 {
            format!("{:>6}", n)
        } else {
            format!("{:>5}{}", n, units[index])
        }
    }
}
