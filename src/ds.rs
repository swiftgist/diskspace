use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process;
use std::sync;

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

pub struct FilesystemDevice {
    pub enabled: bool,
    pub device: u64,
}

impl FilesystemDevice {
    pub fn new() -> FilesystemDevice {
        FilesystemDevice {
            enabled: false,
            device: 0,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get(&mut self, _path: &PathBuf) -> u64 {
        0
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get(&mut self, path: &PathBuf) -> u64 {
        if self.enabled {
            match path.metadata() {
                Err(_) => 0,
                Ok(metadata) => metadata.st_dev(),
            }
        } else {
            0
        }
    }
}

/// Traverse
///
/// Creats a final BTreeMap of all anchors
pub fn traverse(anchors: &Vec<String>, matches: &ArgMatches) -> BTreeMap<String, u64> {
    let mut ve = VerboseErrors::new();
    let mut fd = FilesystemDevice::new();
    let mut diskspace = BTreeMap::new();

    ve.verbose = matches.occurrences_of("verbose") > 0;
    fd.enabled = matches.occurrences_of("one-filesystem") > 0;

    for dir in anchors {
        let mut map = match visit(PathBuf::from(dir), &mut ve, &mut fd) {
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
            Ok(map) => map,
        };
        diskspace.append(&mut map);
    }

    diskspace
}

/// Visit
///
/// Recursively search directories and returns BTreeMaps containing pathnames and
/// filesizes.
pub fn visit(
    path: PathBuf,
    ve: &mut VerboseErrors,
    fd: &mut FilesystemDevice,
) -> Result<BTreeMap<String, u64>, DSError> {
    let mut map = BTreeMap::new();

    if path.is_dir() {
        let anchor_device = fd.get(&path);

        let contents = match fs::read_dir(&path) {
            Ok(contents) => contents,
            Err(err) => {
                ve.display(&path, err);
                return Ok(map);
            }
        };
        let mut total: u64 = 0;
        for entry in contents {
            let child_path = entry.unwrap().path();

            if anchor_device != fd.get(&child_path) {
                continue;
            }
            if symlink_or_error(&path, ve) {
                continue;
            }
            let mut child = match visit(child_path.to_owned(), ve, fd) {
                Ok(child) => child,
                Err(_) => {
                    return Ok(map);
                }
            };

            for (key, value) in child.iter() {
                let pathname = Path::new(key);
                let leaf = pathname.strip_prefix(&path).unwrap().to_string_lossy();
                if !leaf.contains('/') {
                    total += value; // Add immediate children
                }
            }

            map.append(&mut child);
        }
        map.insert(path.to_string_lossy().to_string(), total);
    } else {
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
        map.insert(path.to_string_lossy().to_string(), filesize);
    }

    Ok(map)
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

    //    #[test]
    //    fn increment_err() {
    //        let anchor = PathBuf::from("/tmp");
    //        let mds = Mutex::new(BTreeMap::new());
    //        let path = PathBuf::from("/tmp/does_not_exist");
    //        let mut ve = VerboseErrors::new();
    //
    //        mds.lock()
    //            .unwrap()
    //            .insert("/tmp/does_not_exist".to_string(), 0 as u64);
    //        let result = increment(anchor, &mds, path, &mut ve).ok();
    //        assert_eq!(result, Some(()));
    //        assert_eq!(mds.lock().unwrap().get("/tmp/does_not_exist").unwrap(), &0);
    //    }

    #[test]
    fn symlink_err() {
        let path = PathBuf::from("/tmp/does_not_exist");
        let mut ve = VerboseErrors::new();
        assert_eq!(symlink_or_error(&path, &mut ve), true);
    }

    #[test]
    fn fmt_dserror() {
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

    #[cfg(target_os = "linux")]
    #[test]
    fn filesystem_device_disabled() {
        let mut fd = FilesystemDevice::new();
        let result = fd.get(&PathBuf::from("/tmp"));
        assert_eq!(result, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn filesystem_device_enabled() {
        let mut fd = FilesystemDevice::new();
        fd.enabled = true;
        let result = fd.get(&PathBuf::from("/tmp"));
        assert_ne!(result, 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn filesystem_device_enabled() {
        let mut fd = FilesystemDevice::new();
        fd.enabled = true;
        let result = fd.get(&PathBuf::from("/Users"));
        assert_eq!(result, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn filesystem_device_error() {
        let mut fd = FilesystemDevice::new();
        fd.enabled = true;
        let result = fd.get(&PathBuf::from("/doesnotexist"));
        assert_eq!(result, 0);
    }
}
