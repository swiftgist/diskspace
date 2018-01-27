
mod ds;
use ds::diskspace;
// use ds::diskspace::traverse;
// use ds::diskspace::report;

fn main() {
    let mut directories = Vec::new();

    directories.push("./".to_string());

    let disk_space = diskspace::traverse(directories);

    diskspace::report(disk_space);

}

    // let start = Path::new("./");
    // println!("{:?}", start)
//    disk_space = recurse(start);
//
//
//    fn recurse(pathname: Path) -> BTreeMap<Path, DiskEntry> {
//        let mut disk_space: BTreeMap<Path, DiskEntry> = BTreeMap::new();
//        let path = Path::new(pathname).read_dir().unwrap();
//        for entry in path {
//            if let Ok(entry) = entry {
//                let metadata = entry.metadata().expect("metadata call failed");
//                if entry.path().is_dir() {
//                    let ep = entry.path();
//                    recurse(ep);
//                }
//                println!("{:?} {:?} {}", metadata.st_size(), entry.path(), entry.path().is_dir());
//                //let disk_entry = DiskEntry { size: metadata.st_size(), count: 0 };
//                let de = DiskEntry { size: 10, count: 1 };
//                let ep = entry.path().clone();
//                disk_space.insert(ep, de);
//            }
//        }
//        disk_space
//    }

// fn traverse(mut directories: Vec<String>) -> BTreeMap<String, u64> {
//     let mut disk_space: BTreeMap<String, u64> = BTreeMap::new();
// 
//     while directories.len() != 0 {
//         let dir = directories.pop().unwrap();
//         // println!("{:?}", dir);
//         let path = Path::new(&dir).read_dir().unwrap();
//         for entry in path {
//             if let Ok(entry) = entry {
//                 let pathname = entry.path().to_str().unwrap().to_string();
//                 if entry.path().is_dir() {
//                     directories.push(pathname);
//                 } else {
//                     let metadata = entry.metadata().expect("metadata call failed");
//                     disk_space.insert(pathname, metadata.st_size());
//                     let parent = entry.path().parent().unwrap().to_str().unwrap().to_string();
//                     let mut size: u64 = 0;
//                     if disk_space.contains_key(&parent) {
//                         size = *disk_space.get(&parent).unwrap();
//                     }
//                     size += metadata.st_size();
//                     disk_space.insert(parent, size);
//                 }
//             }
//         }
//     }
//     disk_space
// 
// }
// 
// fn report(disk_space: BTreeMap<String, u64>) {
// 
//     let mut sorted = Vec::from_iter(disk_space);
//     sorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
// 
//     let mut index = 0;
//     for (filename, size) in sorted {
//         if index > 19 { 
//             break;
//         }
//         println!("{} {}", simple_units(size), filename);
//         index += 1;
//     }
// 
// }
// 
// fn simple_units(number: u64) -> String {
//     let units = [ " ", "K", "M", "G", "T", "P" ];
//     let mut index = 0;
//     let mut n = number;
//     while n > 1024 {
//         index += 1;
//         n = n/1024;
//     }
//     if index == 0 {
//         format!("{:>6}", n)
//     } else {
//         format!("{:>5}{}", n, units[index])
//     }
// }
