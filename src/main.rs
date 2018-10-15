extern crate clap;
use clap::{App, Arg};
mod ds;

fn main() {
    let matches = App::new("DiskSpace")
        .version("0.1.0")
        .author("Eric Jackson <swiftgist@gmail.com>")
        .about("Displays disk space usage")
        .arg(Arg::with_name("all").short("a").long("all").help(
            "display all entries",
        ))
        .arg(Arg::with_name("reverse").short("r").long("reverse").help(
            "display entries descending",
        ))
        .arg(Arg::with_name("directory").index(1).help("start location"))
        .get_matches();

    let mut directories = Vec::new();

    let result = matches.value_of("directory");
    if let Some(start) = result {
        directories.push(start.to_string());
    } else {
        directories.push("./".to_string());
    }

    let disk_space = ds::traverse(directories);

    ds::report(disk_space, &matches);
}
