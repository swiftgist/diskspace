use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::process;
use std::sync;
use std::sync::Mutex;

pub enum DSError {
    IO(io::Error),
    Mutex,
}

impl fmt::Display for DSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DSError::IO(err) => write!(f, "{}", err),
            DSError::Mutex => write!(f, "Mutex poisoned"),
        }
    }
}

impl From<io::Error> for DSError {
    fn from(err: io::Error) -> DSError {
        DSError::IO(err)
    }
}

impl<T> From<sync::PoisonError<T>> for DSError {
    fn from(_: sync::PoisonError<T>) -> DSError {
        DSError::Mutex
    }
}

pub fn traverse(anchors: &Vec<String>, _matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut mds = Mutex::new(BTreeMap::new());

    for dir in anchors {
        match visit_dirs(PathBuf::from(dir), &mut mds) {
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
            _ => (),
        }
    }

    let disk_space = mds.lock().ok().unwrap().clone();
    disk_space
}

pub fn visit_dirs(dir: PathBuf, mds: &mut Mutex<BTreeMap<String, u64>>) -> Result<(), DSError> {
    if dir.is_dir() {
        let anchor = dir.to_owned();
        let contents = match fs::read_dir(&dir) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("{} {}", dir.to_string_lossy().to_string(), err);
                return Ok(());
            }
        };
        for entry in contents {
            let entry = entry.unwrap();
            let path = entry.path();

            match fs::symlink_metadata(&path) {
                Ok(metadata) => if metadata.file_type().is_symlink() {
                    continue;
                }
                Err(err) => {
                    eprintln!("{} {}", path.to_string_lossy().to_string(), err);
                    continue;
                }
            }
            if path.is_dir() {
                visit_dirs(path.to_owned(), mds)?;
            } else {
                increment(anchor.to_owned(), &mds, path)?;
            }
        }
    }
    Ok(())
}

fn increment(anchor: PathBuf, mds: &Mutex<BTreeMap<String, u64>>, path: PathBuf) -> Result<(), DSError> {
#[cfg(target_os = "linux")]
    let filesize = path.metadata()?.st_size();
#[cfg(target_os = "windows")]
    let filesize = match path.metadata() {
        Ok(metadata) => metadata.file_size(),
        Err(err) => {
            eprintln!("{} {}", path.to_string_lossy().to_string(), err);
            0
        }
    };
    for ancestor in path.ancestors() {
        let ancestor_path = ancestor.to_string_lossy().to_string();
        *mds.lock()?.entry(ancestor_path).or_insert(0) += filesize;
        if anchor == ancestor {
            break;
        }
    }
    Ok(())
}
