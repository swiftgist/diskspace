
use std::path::Path;
use std::os::linux::fs::MetadataExt;

fn main() {
    let start = Path::new("./");
    recurse(&start);
}


fn recurse(pathname: &Path) {
    let path = Path::new(pathname).read_dir().unwrap();
    for entry in path {
        if let Ok(entry) = entry {
            let metadata = entry.metadata().expect("metadata call failed");
            if entry.path().is_dir() {
                let start = entry.path();
                recurse(&start);
            }
            println!("{:?} {:?} {}", metadata.st_size(), entry.path(), entry.path().is_dir());
        }
    }
}
