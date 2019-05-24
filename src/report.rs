#[cfg(test)]
use clap::App;
use clap::ArgMatches;
extern crate colored;
use self::colored::*;
use std::env;
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
    let end = if matches.occurrences_of("all") == 0 && sorted.len() > 20 {
        20
    } else {
        sorted.len()
    };

    let section = if matches.occurrences_of("reverse") == 0 {
        sorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
        &sorted[0..end]
    } else {
        sorted.sort_by(|&(_, a), &(_, b)| a.cmp(&b));
        &sorted[(sorted.len() - end)..]
    };

    for &(ref filename, size) in section {
        writeln!(out, "{} {}", color(size, matches), filename);
    }
}

fn color(number: u64, matches: &ArgMatches) -> String {
    match env::var_os("TERM") {
        None => simple_units(number),
        Some(term) => {
            match term.as_os_str().to_str().unwrap() {
                "cygwin" => simple_units(number).cyan().bold().to_string(),
                _ => {
                    match matches.value_of("color") {
                        Some("black") => simple_units(number).black().bold().to_string(),
                        Some("red") => simple_units(number).red().bold().to_string(),
                        Some("green") => simple_units(number).green().bold().to_string(),
                        Some("yellow") => simple_units(number).yellow().bold().to_string(),
                        Some("blue") => simple_units(number).blue().bold().to_string(),
                        Some("magenta") => simple_units(number).magenta().bold().to_string(),
                        Some("cyan") => simple_units(number).cyan().bold().to_string(),
                        Some("white") => simple_units(number).white().bold().to_string(),
                        Some("none") => simple_units(number),
                        _ => simple_units(number).yellow().bold().to_string(),
                    }
                }
            }
        }
    }
}

/// Convert number to human friendly format
///
/// Divide successively by 1024 and append the correct suffix
fn simple_units(number: u64) -> String {
    let units = [" ", "K", "M", "G", "T", "P"];
    let index: usize = (number as f64).log(1024.0).trunc() as usize;
    let n = number / 1024u64.pow(index as u32);

    if index == 0 || index > 5 {
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
    fn report_short() {
        let mut data = BTreeMap::new();
        data.insert("path/to/fileA".to_string(), 2048 as u64);
        data.insert("path/to/fileB".to_string(), 1024 as u64);

        let mut out = Vec::new();
        let matches = App::new("DiskSpace").get_matches();
        report_stream(&mut out, data, &matches);
        assert_eq!(
            out,
            format!(
                "{} path/to/fileA\n{} path/to/fileB\n",
                "    2K".yellow().bold(),
                "    1K".yellow().bold()
            )
            .as_bytes()
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
            format!(
                "{} path/to/fileA
{} path/to/fileB
{} path/to/fileC
{} path/to/fileD
{} path/to/fileE
{} path/to/fileF
{} path/to/fileG
{} path/to/fileH
{} path/to/fileI
{} path/to/fileJ
{} path/to/fileK
{} path/to/fileL
{} path/to/fileM
{} path/to/fileN
{} path/to/fileO
{} path/to/fileP
{} path/to/fileQ
{} path/to/fileR
{} path/to/fileS
{} path/to/fileT
",
                "    2K".yellow().bold(),
                "    1K".yellow().bold(),
                "  1023".yellow().bold(),
                "  1022".yellow().bold(),
                "  1021".yellow().bold(),
                "  1020".yellow().bold(),
                "  1019".yellow().bold(),
                "  1018".yellow().bold(),
                "  1017".yellow().bold(),
                "  1016".yellow().bold(),
                "  1015".yellow().bold(),
                "  1014".yellow().bold(),
                "  1013".yellow().bold(),
                "  1012".yellow().bold(),
                "  1011".yellow().bold(),
                "  1010".yellow().bold(),
                "  1009".yellow().bold(),
                "  1008".yellow().bold(),
                "  1007".yellow().bold(),
                "  1006".yellow().bold()
            )
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
