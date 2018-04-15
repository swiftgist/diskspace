
//pub mod diskspace {
use std::iter::FromIterator;
use std::collections::BTreeMap;
//use std::path::Path;
use std::fs;
use std::os::linux::fs::MetadataExt;

pub fn traverse(mut directories: Vec<String>) -> BTreeMap<String, u64> {
    let mut disk_space: BTreeMap<String, u64> = BTreeMap::new();

    while directories.len() != 0 {
        let dir = directories.pop().unwrap();
        // println!("{:?}", dir);
        // let path = Path::new(&dir).read_dir();
        let result = fs::symlink_metadata(&dir);
        if let Ok(metadata) = result {
            // println!("{} {:?}", &dir, metadata.file_type().is_symlink());
            if metadata.file_type().is_symlink() {
                continue;
            }
        }
        let result = fs::read_dir(&dir);
        if let Ok(path) = result {
            process_path(path, &mut directories, &mut disk_space);
        } else {
            println!("Problem with  {}", dir);
        }
    }
    disk_space

}

fn process_path(path: fs::ReadDir,directories: &mut Vec<String>, disk_space: &mut BTreeMap<String, u64>) {

    for result in path {
        if let Ok(entry) = result {
            let epathname = entry.path();
            let rpathname = epathname.to_str();
            if let Some(rpathname) = rpathname {
                let pathname = rpathname.to_string();
                if entry.path().is_dir() {
                    directories.push(pathname);
                } else {
                    process_file(entry, pathname, disk_space)
                }
            }
        }
    }

}

fn process_file(entry: fs::DirEntry, pathname: String, disk_space: &mut BTreeMap<String, u64>) {
    let result = entry.metadata();
    if let Ok(metadata) = result {
        disk_space.insert(pathname, metadata.st_size());
        let entry_path = entry.path();
        let option = entry_path.parent();
        if let Some(rparent) = option {
            let option = rparent.to_str();
            if let Some(sparent) = option {
                increment(sparent, &metadata, disk_space)
            }
        }
    }

}

fn increment(sparent: &str, metadata: &fs::Metadata, disk_space: &mut BTreeMap<String, u64>) {
    let parent = sparent.to_string();
    let mut size: u64 = 0;
    if disk_space.contains_key(&parent) {
        size = *disk_space.get(&parent).unwrap();
    }
    size += metadata.st_size();
    disk_space.insert(parent, size);
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

fn simple_units(number: u64) -> String {
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

#[test]
fn simple_units_bytes() {
    assert_eq!(simple_units(100), "   100");
}

#[test]
fn simple_units_kbytes() {
    assert_eq!(simple_units(1025), "    1K");
}

#[test]
fn simple_units_kbytes_long() {
    assert_eq!(simple_units(1025000), " 1000K");
}

#[test]
fn simple_units_mbytes() {
    assert_eq!(simple_units(2_200_000), "    2M");
}
// }

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

}

