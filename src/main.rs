extern crate clap;
mod cli;
mod ds;
mod report;

use crate::ds::DSGroup;

fn main() {
    let matches = cli::get_matches();
    let anchors: Vec<_> = cli::get_dirs(&matches);
    let mut group = DSGroup::new();

    let disk_space = group.calculate(&anchors, &matches);
    report::report(disk_space, &matches);
}
