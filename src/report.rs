#[cfg(test)]
use clap::App;
use clap::ArgMatches;
use std::collections::BTreeMap;
use std::io;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
use std::iter::FromIterator;

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
    let index = (number as f64).log(1024.0).trunc() as u32;
    let n = number / 1024u64.pow(index);

    if index == 0 {
        format!("{:>6}", n)
    } else {
        format!("{:>5}{}", n, units[index as usize])
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;

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
            "    2K path/to/fileA\n    1K path/to/fileB\n".as_bytes()
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
    1K path/to/fileB
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
".as_bytes()
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
