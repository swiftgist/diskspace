
mod ds;

fn main() {
    let mut directories = Vec::new();

    directories.push("./".to_string());

    let disk_space = ds::traverse(directories);

    ds::report(disk_space);

}
