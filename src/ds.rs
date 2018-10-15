
use clap::ArgMatches;
use std::io;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
use std::iter::FromIterator;
use std::collections::BTreeMap;
use std::os::linux::fs::MetadataExt;
use std::fs;
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use clap::App;

/// Traverse
///
/// Process each directory.  Skip symlinks.
pub fn traverse(mut directories: Vec<String>) -> BTreeMap<String, u64> {
    let mut disk_space: BTreeMap<String, u64> = BTreeMap::new();

    while directories.len() != 0 {
        let dir = directories.pop().unwrap();
        let result = fs::symlink_metadata(&dir);
        if let Ok(metadata) = result {
            if metadata.file_type().is_symlink() {
                continue;
            }
        }
        let result = fs::read_dir(&dir);
        if let Ok(path) = result {
            process_path(path, &mut directories, &mut disk_space);
        } else {
            println!("Cannot read directory {}", dir);
        }
    }
    disk_space

}

/// Process Path
///
/// Find all directories and files in a path.  Append directories to a vector
/// and process each file
fn process_path(
    path: fs::ReadDir,
    directories: &mut Vec<String>,
    disk_space: &mut BTreeMap<String, u64>,
) {

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

/// Processs file
///
/// Add the file to the hash with its size and increment the parent by that size
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

/// Add the file size to the parent entry
///
/// Find the parent entry in the hash, record the current size and increment
/// by the st_size of the file.
fn increment(sparent: &str, metadata: &fs::Metadata, disk_space: &mut BTreeMap<String, u64>) {
    let mut size: u64 = 0;
    if disk_space.contains_key(sparent) {
        size = *disk_space.get(sparent).unwrap();
    }
    size += metadata.st_size();
    disk_space.insert(sparent.to_string(), size);
}

/// Generate a text report
///
/// Send report to stdout
pub fn report(disk_space: BTreeMap<String, u64>, matches: &ArgMatches) {

    report_stream(&mut io::stdout(), disk_space, matches)

}

/// Generate a text report
///
/// Sort the entries by size and output the top 20
#[allow(unused_must_use)]
pub fn report_stream(out: &mut io::Write, disk_space: BTreeMap<String, u64>, matches: &ArgMatches) {

    let mut sorted = Vec::from_iter(disk_space);
    let end;
    if matches.occurrences_of("all") == 0 {
        end = if sorted.len() < 20 { sorted.len() } else { 20 };
    } else {
        end = sorted.len();
    }

    let section;
    if matches.occurrences_of("reverse") == 0 {
        sorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
        section = &sorted[0..end];
    } else {
        sorted.sort_by(|&(_, a), &(_, b)| a.cmp(&b));
        section = &sorted[(sorted.len() - end)..];
    }

    // for &(ref filename, size) in &sorted[0..end] {
    for &(ref filename, size) in section {
        writeln!(out, "{} {}", simple_units(size), filename);
    }

}

/// Convert number to human friendly format
///
/// Divide successively by 1024 and append the correct suffix
fn simple_units(number: u64) -> String {
    let units = [" ", "K", "M", "G", "T", "P"];
    let mut index = 0;
    let mut n = number;
    while n > 1024 {
        index += 1;
        n = n / 1024;
    }
    if index == 0 {
        format!("{:>6}", n)
    } else {
        format!("{:>5}{}", n, units[index])
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        // No idea how to mock fs access yet
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(b"Hello World!").unwrap();
        let metadata = fs::metadata("foo.txt").unwrap();

        let parent = String::from("path/to/file");
        let mut data = BTreeMap::new();
        data.insert("path/to/file".to_string(), 0 as u64);

        increment(&parent, &metadata, &mut data);
        fs::remove_file("foo.txt");

        assert_eq!(data["path/to/file"], 12)
    }


    #[test]
    fn report_short() {
        let mut data = BTreeMap::new();
        data.insert("path/to/fileA".to_string(), 2048 as u64);
        data.insert("path/to/fileB".to_string(), 1024 as u64);

        let mut out = Vec::new();
        let matches = App::new("DiskSpace").get_matches();
        report_stream(&mut out, data, &matches);
        assert_eq!(
            out,
            "    2K path/to/fileA\n  1024 path/to/fileB\n".as_bytes()
        )

    }

    #[test]
    fn report_long() {
        let mut data = BTreeMap::new();
        data.insert("path/to/fileA".to_string(), 2048 as u64);
        data.insert("path/to/fileB".to_string(), 1024 as u64);
        data.insert("path/to/fileC".to_string(), 1023 as u64);
        data.insert("path/to/fileD".to_string(), 1022 as u64);
        data.insert("path/to/fileE".to_string(), 1021 as u64);
        data.insert("path/to/fileF".to_string(), 1020 as u64);
        data.insert("path/to/fileG".to_string(), 1019 as u64);
        data.insert("path/to/fileH".to_string(), 1018 as u64);
        data.insert("path/to/fileI".to_string(), 1017 as u64);
        data.insert("path/to/fileJ".to_string(), 1016 as u64);
        data.insert("path/to/fileK".to_string(), 1015 as u64);
        data.insert("path/to/fileL".to_string(), 1014 as u64);
        data.insert("path/to/fileM".to_string(), 1013 as u64);
        data.insert("path/to/fileN".to_string(), 1012 as u64);
        data.insert("path/to/fileO".to_string(), 1011 as u64);
        data.insert("path/to/fileP".to_string(), 1010 as u64);
        data.insert("path/to/fileQ".to_string(), 1009 as u64);
        data.insert("path/to/fileR".to_string(), 1008 as u64);
        data.insert("path/to/fileS".to_string(), 1007 as u64);
        data.insert("path/to/fileT".to_string(), 1006 as u64);
        data.insert("path/to/fileU".to_string(), 1005 as u64);

        let mut out = Vec::new();
        let matches = App::new("DiskSpace").get_matches();
        report_stream(&mut out, data, &matches);
        assert_eq!(
            out,
            "    2K path/to/fileA
  1024 path/to/fileB
  1023 path/to/fileC
  1022 path/to/fileD
  1021 path/to/fileE
  1020 path/to/fileF
  1019 path/to/fileG
  1018 path/to/fileH
  1017 path/to/fileI
  1016 path/to/fileJ
  1015 path/to/fileK
  1014 path/to/fileL
  1013 path/to/fileM
  1012 path/to/fileN
  1011 path/to/fileO
  1010 path/to/fileP
  1009 path/to/fileQ
  1008 path/to/fileR
  1007 path/to/fileS
  1006 path/to/fileT
"
                .as_bytes()
        )

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

}
