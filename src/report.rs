#![allow(dead_code)] // don't like this

#[cfg(test)]
use clap::App;
use clap::ArgMatches;
use std::collections::BTreeMap;
extern crate colored;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;

pub struct ReportSettings {
    pub all: bool,
    pub reverse: bool,
    pub lines: usize,
    pub exclude: Vec<String>,
}

impl ReportSettings {
    pub fn new() -> ReportSettings {
        ReportSettings {
            all: false,
            reverse: false,
            lines: 20,
            exclude: Vec::new(),
        }
    }

    pub fn settings(&mut self, matches: &ArgMatches) {
        self.all = matches.occurrences_of("all") > 0;
        self.reverse = matches.occurrences_of("reverse") > 0;

        if let Some(lines) = matches.value_of("lines") {
            self.lines = match lines.to_string().parse() {
                Err(err) => {
                    eprintln!("Check lines option: {}", err);
                    self.lines
                }
                Ok(lines) => lines,
            }
        }

        if let Some(exclude) = matches.values_of("exclude") {
            self.exclude = exclude.map(|x| x.to_string()).collect();
        }
    }
}

// /// Report
// ///
// /// Send report to stdout
// pub fn report(disk_space: BTreeMap<String, u64>, matches: &ArgMatches) {
//     report_stream(&mut io::stdout(), disk_space, matches)
// }
//
// /// Report_Stream
// ///
// /// Sort the entries by size and output the top 20
// #[allow(unused_must_use)]
// pub fn report_stream(
//     out: &mut dyn io::Write,
//     mut disk_space: BTreeMap<String, u64>,
//     matches: &ArgMatches,
// ) {
//     let mut rs = ReportSettings::new();
//     rs.settings(matches);
//     if !rs.exclude.is_empty() {
//         disk_space = exclude(&rs, disk_space);
//     }
//
//     let mut unsorted = Vec::from_iter(disk_space);
//     let end = endpoint(&rs, unsorted.len());
//
//     let sorted = if rs.reverse {
//         unsorted.sort_by(|&(_, a), &(_, b)| a.cmp(&b));
//         &unsorted[(unsorted.len() - end)..]
//     } else {
//         unsorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
//         &unsorted[0..end]
//     };
//
//     for &(ref filename, size) in sorted {
//         writeln!(out, "{} {}", color(size, matches), filename);
//     }
// }

pub fn endpoint(rs: &ReportSettings, length: usize) -> usize {
    if !rs.all && length > rs.lines {
        rs.lines
    } else {
        length
    }
}

pub fn exclude(rs: &ReportSettings, disk_space: BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let mut tmp = BTreeMap::new();
    let mut include = true;
    for filename in disk_space.keys() {
        for exclusion in &rs.exclude {
            if filename.contains(exclusion) {
                include = false;
                break;
            }
        }
        if include {
            tmp.insert(filename.to_string(), *disk_space.get(filename).unwrap());
        } else {
            include = true;
        }
    }
    tmp
}

// /// Color
// ///
// /// Returns a string that will contain colored unit output if the
// /// TERM environment variable is set.  Defaults to yellow on Linux and
// /// cyan on Windows(cygwin).  Color preference specified as a command
// /// line option.
// fn color(number: u64, matches: &ArgMatches) -> String {
//     match env::var_os("TERM") {
//         None => simple_units(number),
//         Some(term) => match term.as_os_str().to_str().unwrap() {
//             "cygwin" => simple_units(number).cyan().bold().to_string(),
//             _ => match matches.value_of("color") {
//                 Some("black") => simple_units(number).black().bold().to_string(),
//                 Some("red") => simple_units(number).red().bold().to_string(),
//                 Some("green") => simple_units(number).green().bold().to_string(),
//                 Some("yellow") => simple_units(number).yellow().bold().to_string(),
//                 Some("blue") => simple_units(number).blue().bold().to_string(),
//                 Some("magenta") => simple_units(number).magenta().bold().to_string(),
//                 Some("cyan") => simple_units(number).cyan().bold().to_string(),
//                 Some("white") => simple_units(number).white().bold().to_string(),
//                 Some("none") => simple_units(number),
//                 _ => simple_units(number).yellow().bold().to_string(),
//             },
//         },
//     }
// }

/// Simple_Units
///
/// Convert number to human friendly format
pub fn simple_units(number: u64) -> String {
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

    #[test]
    fn settings_defaults() {
        let args = vec!["ds"];
        let matches = App::new("DiskSpace").get_matches_from(args);
        let mut rs = ReportSettings::new();
        rs.settings(&matches);
        assert_eq!(rs.all, false);
        assert_eq!(rs.reverse, false);
        assert_eq!(rs.lines, 20);
    }
}
