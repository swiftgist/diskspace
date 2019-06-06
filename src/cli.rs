extern crate clap;
use clap::{App, Arg, ArgMatches};

pub fn get_matches() -> ArgMatches<'static> {
    App::new("DiskSpace")
        .version("0.1.0")
        .author("Eric Jackson <swiftgist@gmail.com>")
        .about("Displays disk space usage")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("display all entries"),
        )
        .arg(
            Arg::with_name("color")
                .short("c")
                .long("color")
                .value_name("COLOR")
                .help("set to black, red, green, yellow, blue, magenta, cyan, white or none")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("display skipped entries"),
        )
        .arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("display entries descending"),
        )
        .arg(
            Arg::with_name("directory")
                .min_values(0)
                .help("start location"),
        )
        .get_matches()
}

pub fn get_dirs(matches: &ArgMatches) -> Vec<String> {
    match matches.values_of("directory") {
        Some(start) => start.map(|x| x.to_string()).collect(),
        None => vec!["./".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check() {
        let _ = get_matches();
        // tarpaulin coverage
    }

    #[test]
    fn dirs() {
        let args = vec!["ds", "/tmp"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("directory").min_values(0))
            .get_matches_from(args);

        let result = get_dirs(&matches);

        assert_eq!(result, vec!["/tmp".to_string()]);
    }

    #[test]
    fn dirs_default() {
        let args = vec!["ds"];
        let matches = App::new("DiskSpace")
            .arg(Arg::with_name("directory").min_values(0))
            .get_matches_from(args);

        let result = get_dirs(&matches);

        assert_eq!(result, vec!["./".to_string()]);
    }

}
