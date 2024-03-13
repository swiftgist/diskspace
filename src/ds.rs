use clap::ArgMatches;
use std::collections::btree_map::Entry;
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
// use std::time::Instant;

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

/// FilesystemDevice
///
/// Linux supports filesystems independent of directory paths.  Support restricting
/// calculations to a single filesystem.
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
                Ok(metadata) => metadata.st_dev(),
                Err(_) => 0,
            }
        } else {
            0
        }
    }
}

/// DSGroup
///
/// Data structures for calculations:
///   inodes: file inodes
///   dirs: map of directory paths and list of children
///   sizes: final collection of sizes for all files and directories
pub struct DSGroup {
    pub ve: VerboseErrors,
    pub fd: FilesystemDevice,
    pub inodes: BTreeMap<u64, bool>,
    pub dirs: BTreeMap<PathBuf, Vec<PathBuf>>,
    pub sizes: BTreeMap<String, u64>,
}

impl DSGroup {
    pub fn new() -> DSGroup {
        DSGroup {
            ve: VerboseErrors::new(),
            fd: FilesystemDevice::new(),
            inodes: BTreeMap::new(),
            dirs: BTreeMap::new(),
            sizes: BTreeMap::new(),
        }
    }

    /// calculate
    ///
    /// Check command line options.  Calculate file and directory size.  Append to map.
    pub fn calculate(
        &mut self,
        anchors: &Vec<String>,
        matches: &ArgMatches,
    ) -> BTreeMap<String, u64> {
        self.ve.verbose = matches.occurrences_of("verbose") > 0;
        self.fd.enabled = matches.occurrences_of("one-filesystem") > 0;

        let mut diskspace = BTreeMap::new();

        for dir in anchors {
            // let start = Instant::now();
            let _ = match self.traverse(PathBuf::from(dir)) {
                Ok(list) => list,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            };
            // let duration = start.elapsed();
            // if duration.as_millis() > 100 {
            //     println!("Time elapsed for files is: {:?}", duration);
            // }

            // let start = Instant::now();
            self.calculate_dirsize();
            // let duration = start.elapsed();
            // if duration.as_millis() > 100 {
            //     println!("Time elapsed for dirs is: {:?}", duration);
            // }

            diskspace.append(&mut self.sizes);
        }
        diskspace
    }

    /// traverse
    ///
    /// Recursively evaluate files and collect children for directories. Skip symlinks.
    fn traverse(&mut self, path: PathBuf) -> Result<(), DSError> {
        if path.is_dir() {
            let contents = match fs::read_dir(&path) {
                Ok(contents) => contents,
                Err(_) => {
                    return Ok(());
                }
            };

            let anchor_device = self.fd.get(&path);
            let mut children = vec![];

            for entry in contents {
                let child_path = entry.unwrap().path();

                if anchor_device != self.fd.get(&child_path) {
                    continue;
                }

                if self.is_symlink(&child_path) {
                    continue;
                }

                children.push(child_path.clone());

                let _ = match self.traverse(child_path) {
                    Ok(child) => child,
                    Err(_) => return Ok(()),
                };
            }
            self.dirs.insert(path, children);
        } else {
            self.record_filesize(&path);
        }

        Ok(())
    }

    /// record_filesize
    ///
    /// Retrieve the inode.  Retrieve the filesize if the inode is absent. Add entry
    /// to sizes.  Skips hard links.
    fn record_filesize(&mut self, path: &PathBuf) {
        let inode = match path.metadata() {
            #[cfg(target_os = "linux")]
            Ok(metadata) => metadata.st_ino(),
            #[cfg(target_os = "windows")]
            Ok(metadata) => metadata.st_ino(),
            Err(err) => {
                self.ve.display(&path, err);
                0
            }
        };

        let filesize = match self.inodes.entry(inode) {
            Entry::Vacant(_o) => {
                self.inodes.insert(inode, false);

                match path.metadata() {
                    #[cfg(target_os = "linux")]
                    Ok(metadata) => metadata.st_size(),
                    #[cfg(target_os = "windows")]
                    Ok(metadata) => metadata.file_size(),
                    Err(err) => {
                        self.ve.display(&path, err);
                        0
                    }
                }
            }
            Entry::Occupied(_o) => 0,
        };

        if filesize > 0 {
            self.sizes
                .insert(path.to_string_lossy().to_string(), filesize);
        }
    }

    /// calculate_dirsize
    ///
    /// Reverse the keys of the map and sum the children.  Hard links and symlinks
    /// are omitted.
    fn calculate_dirsize(&mut self) {
        for dir in self.dirs.keys().rev() {
            let mut dirsize: u64 = 0;
            if let Some(children) = self.dirs.get(dir) {
                for child in children {
                    let size = match self.sizes.get(&child.to_string_lossy().to_string()) {
                        Some(size) => *size,
                        None => 0,
                    };
                    dirsize += size;
                }
            }
            self.sizes
                .insert(dir.to_string_lossy().to_string(), dirsize);
        }
    }

    /// is_symlink
    ///
    /// Check if a path is a symlink.  Returns true if path is a symlink
    /// or if the metadata results in an error.
    fn is_symlink(&mut self, path: &PathBuf) -> bool {
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return true;
                }
            }
            Err(err) => {
                self.ve.display(path, err);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

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
        assert_eq!(is_symlink(&path, &mut ve), true);
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

    //    #[test]
    //    fn cast_mutex_error() {
    //        fn nothing() -> DSError {
    //            let err = PoisonError::new(Mutex::new(1));
    //            From::from(err)
    //        }
    //
    //        let result = format!("{}", nothing());
    //        assert_eq!(result, "Mutex poisoned");
    //    }

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
