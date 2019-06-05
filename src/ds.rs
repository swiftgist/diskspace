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

/// Current implementation
/// Expand upon the basic solution from ds4.rs.  Include proper error
/// handling and replace unwrap() with ? where possible.  Increase
/// functionality with additional command line options and windows
/// support.

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

/// VerboseErrors
///
/// Two flags to track whether files with errors should be printed or
/// an informational message to the user.
pub struct VerboseErrors {
    pub verbose: bool,
    once: bool,
}

impl VerboseErrors {
    pub fn new() -> VerboseErrors {
        VerboseErrors {
            verbose: false,
            once: true,
        }
    }

    pub fn display(&mut self, path: &PathBuf, err: io::Error) {
        if self.verbose {
            eprintln!("{} {}", path.to_string_lossy().to_string(), err);
        } else {
            if self.once {
                eprintln!("Use -v to see skipped files");
                self.once = false;
            }
        }
    }
}

/// Traverse
///
/// Creates a Mutex of a BTreeMap and a VerboseErrors.  Supports scanning
/// multiple directories.
pub fn traverse(anchors: &Vec<String>, matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut mds = Mutex::new(BTreeMap::new());
    let mut ve = VerboseErrors::new();

    ve.verbose = matches.occurrences_of("verbose") > 0;

    for dir in anchors {
        match visit_dirs(PathBuf::from(dir), &mut mds, &mut ve) {
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

/// Visit_Dirs
///
/// Recursively searches a directory and returns Result<>. Ignores
/// directories with errors and symlinks.
pub fn visit_dirs(
    dir: PathBuf,
    mds: &mut Mutex<BTreeMap<String, u64>>,
    ve: &mut VerboseErrors,
) -> Result<(), DSError> {
    if dir.is_dir() {
        let anchor = dir.to_owned();
        let contents = match fs::read_dir(&dir) {
            Ok(contents) => contents,
            Err(err) => {
                ve.display(&dir, err);
                return Ok(());
            }
        };
        for entry in contents {
            let entry = entry.unwrap();
            let path = entry.path();

            if symlink_or_error(&path, ve) {
                continue;
            }
            if path.is_dir() {
                visit_dirs(path.to_owned(), mds, ve)?;
            } else {
                increment(anchor.to_owned(), &mds, path, ve)?;
            }
        }
    }
    Ok(())
}

/// Symlink_or_Error
///
/// Check if a path is a symlink.  Returns true if path is a symlink
/// or if the metadata results in an error.
fn symlink_or_error(path: &PathBuf, ve: &mut VerboseErrors) -> bool {
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return true;
            }
        }
        Err(err) => {
            ve.display(path, err);
            return true;
        }
    }
    false
}

/// Increment
///
/// Finds filesize for Linux and Windows.  Effectively skips files with
/// errors.  Increment the size of the path and all ancestors.
fn increment(
    anchor: PathBuf,
    mds: &Mutex<BTreeMap<String, u64>>,
    path: PathBuf,
    ve: &mut VerboseErrors,
) -> Result<(), DSError> {
    let filesize = match path.metadata() {
        #[cfg(target_os = "linux")]
        Ok(metadata) => metadata.st_size(),
        #[cfg(target_os = "windows")]
        Ok(metadata) => metadata.file_size(),
        Err(err) => {
            ve.display(&path, err);
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

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};
    use std::sync::PoisonError;

    #[test]
    fn display() {
        let mut ve = VerboseErrors::new();
        ve.verbose = false;
        let err = Error::new(ErrorKind::Other, "example");
        assert_eq!(ve.display(&PathBuf::from("/some/path"), err), ());
    }

    #[test]
    fn display_verbose() {
        let mut ve = VerboseErrors::new();
        ve.verbose = true;
        let err = Error::new(ErrorKind::Other, "example");
        assert_eq!(ve.display(&PathBuf::from("/some/path"), err), ());
    }

    #[test]
    fn increment_err() {
        let anchor = PathBuf::from("/tmp");
        let mds = Mutex::new(BTreeMap::new());
        let path = PathBuf::from("/tmp/does_not_exist");
        let mut ve = VerboseErrors::new();

        mds.lock().unwrap().insert("/tmp/does_not_exist".to_string(), 0 as u64);
        let result = increment(anchor, &mds, path, &mut ve).ok();
        assert_eq!(result, Some(()));
        assert_eq!(mds.lock().unwrap().get("/tmp/does_not_exist").unwrap(), &0);
    }

    #[test]
    fn symlink_err() {
        let path = PathBuf::from("/tmp/does_not_exist");
        let mut ve = VerboseErrors::new();
        assert_eq!(symlink_or_error(&path, &mut ve), true);
    }

    #[test]
    fn fmt_dserror(){
        let result = format!("{}", DSError::Mutex);
        assert_eq!(result, "Mutex poisoned");
    }

    #[test]
    fn cast_ioerror() {
        fn nothing() -> DSError {
           let err = Error::new(ErrorKind::Other, "example");
           From::from(err)
        }

        let result = format!("{}", nothing());
        assert_eq!(result, "example");

    }

    #[test]
    fn cast_mutex_error() {
        fn nothing() -> DSError {
           let err = PoisonError::new(Mutex::new(1));
           From::from(err)
        }

        let result = format!("{}", nothing());
        assert_eq!(result, "Mutex poisoned");
    }
}

