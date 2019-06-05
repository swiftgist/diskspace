extern crate clap;
mod cli;
mod ds;
mod report;

fn main() {
    let matches = cli::get_matches();
    let anchors: Vec<_> = cli::get_dirs(&matches);
    let disk_space = ds::traverse(&anchors, &matches);

    report::report(disk_space, &matches);
}
