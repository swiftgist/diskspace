extern crate clap;
mod cli;
mod ds;
mod report;
mod report_gui;

use crate::ds::DSGroup;

fn main() {
    let matches = cli::get_matches();
    let anchors: Vec<_> = cli::get_dirs(&matches);
    let mut group = DSGroup::new();

    let disk_space = group.calculate(&anchors, &matches);
    let _list = report_gui::report_map(disk_space, &matches);
}
