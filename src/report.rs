#[cfg(test)]
use clap::App;
use clap::ArgMatches;
extern crate colored;
use self::colored::*;
use std::collections::BTreeMap;
use std::env;
use std::io;
#[allow(unused_imports)] // method write_all is needed
use std::io::Write;
use std::iter::FromIterator;

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

/// Report
///
/// Send report to stdout
pub fn report(disk_space: BTreeMap<String, u64>, matches: &ArgMatches) {
    report_stream(&mut io::stdout(), disk_space, matches)
}

/// Report_Stream
///
/// Sort the entries by size and output the top 20
#[allow(unused_must_use)]
pub fn report_stream(
    out: &mut io::Write,
    mut disk_space: BTreeMap<String, u64>,
    matches: &ArgMatches,
) {
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

    for &(ref filename, size) in sorted {
        writeln!(out, "{} {}", color(size, matches), filename);
    }
}

fn endpoint(rs: &ReportSettings, length: usize) -> usize {
    if !rs.all && length > rs.lines {
        rs.lines
    } else {
        length
    }
}

fn exclude(rs: &ReportSettings, disk_space: BTreeMap<String, u64>) -> BTreeMap<String, u64> {
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

/// Color
///
/// Returns a string that will contain colored unit output if the
/// TERM environment variable is set.  Defaults to yellow on Linux and
/// cyan on Windows(cygwin).  Color preference specified as a command
/// line option.
fn color(number: u64, matches: &ArgMatches) -> String {
    match env::var_os("TERM") {
        None => simple_units(number),
        Some(term) => match term.as_os_str().to_str().unwrap() {
            "cygwin" => simple_units(number).cyan().bold().to_string(),
            _ => match matches.value_of("color") {
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
            },
        },
    }
}

/// Simple_Units
///
/// Convert number to human friendly format
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
    use clap::Arg;
    use std::env;

    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "linux")]
    #[test]
    fn report_short_reverse() {
        let mut data = BTreeMap::new();
        data.insert("path/to/fileA".to_string(), 2048 as u64);
        data.insert("path/to/fileB".to_string(), 1024 as u64);

        let mut out = Vec::new();
        let args = vec!["ds", "-r"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("reverse").short("r"))
            .get_matches_from(args);
        report_stream(&mut out, data, &matches);
        assert_eq!(
            out,
            format!(
                "{} path/to/fileB\n{} path/to/fileA\n",
                "    1K".yellow().bold(),
                "    2K".yellow().bold(),
            )
            .as_bytes()
        )
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn report_short_exclude() {
        let mut data = BTreeMap::new();
        data.insert("path/to/fileA".to_string(), 2048 as u64);
        data.insert("path/to/fileB".to_string(), 1024 as u64);

        let mut out = Vec::new();
        let args = vec!["ds", "-e", "fileB"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("exclude")
                    .short("e")
                    .min_values(1)
                    .multiple(true),
            )
            .get_matches_from(args);
        report_stream(&mut out, data, &matches);
        assert_eq!(
            out,
            format!("{} path/to/fileA\n", "    2K".yellow().bold(),).as_bytes()
        )
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn report_stdout() {
        let data = BTreeMap::new();
        let matches = App::new("DiskSpace").get_matches();
        report(data, &matches);
    }

    #[cfg(target_os = "linux")]
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

    #[test]
    fn color_black() {
        let args = vec!["ds", "-c", "black"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".black().bold().to_string());
    }

    #[test]
    fn color_red() {
        let args = vec!["ds", "-c", "red"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".red().bold().to_string());
    }

    #[test]
    fn color_green() {
        let args = vec!["ds", "-c", "green"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".green().bold().to_string());
    }

    #[test]
    fn color_yellow() {
        let args = vec!["ds", "-c", "yellow"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".yellow().bold().to_string());
    }

    #[test]
    fn color_blue() {
        let args = vec!["ds", "-c", "blue"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".blue().bold().to_string());
    }

    #[test]
    fn color_magenta() {
        let args = vec!["ds", "-c", "magenta"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".magenta().bold().to_string());
    }

    #[test]
    fn color_cyan() {
        let args = vec!["ds", "-c", "cyan"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".cyan().bold().to_string());
    }

    #[test]
    fn color_white() {
        let args = vec!["ds", "-c", "white"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10".white().bold().to_string());
    }

    #[test]
    fn color_none() {
        let args = vec!["ds", "-c", "none"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("color")
                    .short("c")
                    .value_name("COLOR")
                    .takes_value(true),
            )
            .get_matches_from(args);
        env::set_var("TERM", "xterm-256color");

        let result = color(10, &matches);
        assert_eq!(result, "    10");
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

    #[test]
    fn settings_all() {
        let args = vec!["ds", "-a"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("all").short("a"))
            .get_matches_from(args);
        let mut rs = ReportSettings::new();
        rs.settings(&matches);
        assert_eq!(rs.all, true);
    }

    #[test]
    fn settings_reverse() {
        let args = vec!["ds", "-r"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("reverse").short("r"))
            .get_matches_from(args);
        let mut rs = ReportSettings::new();
        rs.settings(&matches);
        assert_eq!(rs.reverse, true);
    }

    #[test]
    fn settings_lines() {
        let args = vec!["ds", "-n", "10"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("lines").short("n").takes_value(true))
            .get_matches_from(args);
        let mut rs = ReportSettings::new();
        rs.settings(&matches);
        assert_eq!(rs.lines, 10);
    }

    #[test]
    fn settings_lines_invalid_value() {
        let args = vec!["ds", "-n", "apple"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("lines").short("n").takes_value(true))
            .get_matches_from(args);
        let mut rs = ReportSettings::new();
        rs.settings(&matches);
        assert_eq!(rs.lines, 20);
    }

    #[test]
    fn settings_exclude() {
        let args = vec!["ds", "-e", "apple", "pear"];
        let matches = App::new("DiskSpace")
            .arg(
                Arg::with_name("exclude")
                    .short("e")
                    .min_values(1)
                    .multiple(true)
                    .takes_value(true),
            )
            .get_matches_from(args);
        let mut rs = ReportSettings::new();
        rs.settings(&matches);
        assert_eq!(rs.exclude, vec!["apple".to_string(), "pear".to_string()]);
    }

}
