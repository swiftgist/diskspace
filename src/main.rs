extern crate clap;
use clap::{App, Arg};
mod ds;

fn main() {
    let matches = App::new("DiskSpace")
        .version("0.1.0")
        .author("Eric Jackson <swiftgist@gmail.com>")
        .about("Displays disk space usage")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("display all entries"),
        ).arg(
            Arg::with_name("parent")
                .short("p")
                .long("parent")
                .help("include all parent directories"),
        ).arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("display entries descending"),
        ).arg(Arg::with_name("directory").index(1).help("start location"))
        .get_matches();

    let anchor = match matches.value_of("directory") {
        Some(start) => start.to_string(),
        None => "./".to_string(),
    };

    let disk_space = ds::traverse(&anchor, &matches);

    ds::report(disk_space, &matches);
}
