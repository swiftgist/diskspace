use clap::ArgMatches;
extern crate colored;
use std::collections::BTreeMap;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
use std::iter::FromIterator;

use crate::report::*;

pub fn report_map(
    mut disk_space: BTreeMap<String, u64>,
    matches: &ArgMatches,
) -> Vec<(String, String)> {
    let mut rs = ReportSettings::new();
    rs.settings(matches);
    if !rs.exclude.is_empty() {
        disk_space = exclude(&rs, disk_space);
    }

    let mut unsorted = Vec::from_iter(disk_space);
    let end = endpoint(&rs, unsorted.len());

    let sorted = if rs.reverse {
        unsorted.sort_by(|&(_, a), &(_, b)| a.cmp(&b));
        &unsorted[(unsorted.len() - end)..]
    } else {
        unsorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
        &unsorted[0..end]
    };

    let mut list = Vec::new();
    for &(ref filename, size) in sorted {
        list.push((simple_units(size), filename.clone()));
    }
    list
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use clap::App;

    #[cfg(target_os = "linux")]
    #[test]
    fn report_short() {
        let mut data = BTreeMap::new();
        data.insert("path/to/fileA".to_string(), 2048 as u64);
        data.insert("path/to/fileB".to_string(), 1024 as u64);

        let matches = App::new("DiskSpace").get_matches();
        let result = report_map(data, &matches);
        assert_eq!(
            result,
            vec![
                ("    2K".to_string(), "path/to/fileA".to_string()),
                ("    1K".to_string(), "path/to/fileB".to_string())
            ]
        )
    }
}
