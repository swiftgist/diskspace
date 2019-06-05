extern crate clap;
// use clap::{App, Arg};
mod ds;
#[cfg(feature = "multiple")]
mod ds1;
#[cfg(feature = "multiple")]
mod ds2;
#[cfg(feature = "multiple")]
mod ds3;
#[cfg(feature = "multiple")]
mod ds4;
#[cfg(feature = "multiple")]
mod ds5;
mod cli;
mod report;

#[cfg(not(feature = "multiple"))]
fn main() {
    let matches = cli::get_matches();
    let anchors: Vec<_> = cli::get_dirs(&matches);
    let disk_space = ds::traverse(&anchors, &matches);

    report::report(disk_space, &matches);
}

// Build with "cargo build --features multiple" if you wish to
// experiment with the different implementations
#[cfg(feature = "multiple")]
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
        )
        .arg(
            Arg::with_name("parent")
                .short("p")
                .long("parent")
                .help("include all parent directories"),
        )
        .arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("display entries descending"),
        )
        .arg(
            Arg::with_name("first")
                .short("1")
                .help("original implementation"),
        )
        .arg(
            Arg::with_name("second")
                .short("2")
                .help("second implementation"),
        )
        .arg(
            Arg::with_name("third")
                .short("3")
                .help("third implementation"),
        )
        .arg(
            Arg::with_name("fourth")
                .short("4")
                .help("fourth implementation"),
        )
        .arg(
            Arg::with_name("fifth")
                .short("5")
                .help("fifth implementation"),
        )
        .arg(Arg::with_name("directory").index(1).help("start location"))
        .get_matches();

    let anchor = match matches.value_of("directory") {
        Some(start) => start.to_string(),
        None => "./".to_string(),
    };

    if matches.is_present("fifth") {
        let disk_space = ds5::traverse(&anchor, &matches);
        report::report(disk_space, &matches);
    } else if matches.is_present("fourth") {
        let disk_space = ds4::traverse(&anchor, &matches);
        report::report(disk_space, &matches);
    } else if matches.is_present("third") {
        let disk_space = ds3::traverse(&anchor, &matches);
        report::report(disk_space, &matches);
    } else if matches.is_present("second") {
        let disk_space = ds2::traverse(&anchor, &matches);
        report::report(disk_space, &matches);
    } else if matches.is_present("first") {
        let disk_space = ds1::traverse(&anchor, &matches);
        report::report(disk_space, &matches);
    } else {
        let disk_space = ds::traverse(&anchor, &matches);
        report::report(disk_space, &matches);
    }
}
