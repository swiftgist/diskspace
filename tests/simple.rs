extern crate tempdir;
extern crate ds;
extern crate clap;
use clap::App;
use ds::traverse;
use tempdir::TempDir;
use std::collections::BTreeMap;
use std::process::Command;
use std::fs;
use std::os::unix;
use std::io::Write;

#[cfg(test)]
fn setup() {}

#[test]
fn simple() {
    setup();
    let output = Command::new("target/debug/ds")
        .output()
        .expect("failed to execute ds");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("M ./target/debug/ds",));
}

#[test]
fn sample_directories() {
    let mut expected = BTreeMap::new();

    // create directory structure and files
    let tmp_dir = TempDir::new("/tmp/dstest").unwrap();
    let tmppath = tmp_dir.path().to_owned();

    expected.insert(format!("{}/a", tmppath.display()), 165);
    expected.insert(format!("{}/a/b", tmppath.display()), 165);
    expected.insert(format!("{}/a/b/c", tmppath.display()), 165);
    expected.insert(format!("{}/a/b/c/d", tmppath.display()), 165);
    expected.insert(format!("{}/a/b/c/d/sample1", tmppath.display()), 15);
    expected.insert(format!("{}/a/b/c/d/sample2", tmppath.display()), 24);
    expected.insert(format!("{}/a/b/c/d/sample3", tmppath.display()), 33);
    expected.insert(format!("{}/a/b/c/d/sample4", tmppath.display()), 42);
    expected.insert(format!("{}/a/b/c/d/sample5", tmppath.display()), 51);
    expected.insert(format!("{}/a/b/c/sample1", tmppath.display()), 15);
    expected.insert(format!("{}/a/b/c/sample2", tmppath.display()), 24);
    expected.insert(format!("{}/a/b/c/sample3", tmppath.display()), 33);
    expected.insert(format!("{}/a/b/c/sample4", tmppath.display()), 42);
    expected.insert(format!("{}/a/b/c/sample5", tmppath.display()), 51);
    expected.insert(format!("{}/a/b/sample1", tmppath.display()), 15);
    expected.insert(format!("{}/a/b/sample2", tmppath.display()), 24);
    expected.insert(format!("{}/a/b/sample3", tmppath.display()), 33);
    expected.insert(format!("{}/a/b/sample4", tmppath.display()), 42);
    expected.insert(format!("{}/a/b/sample5", tmppath.display()), 51);
    expected.insert(format!("{}/a/sample1", tmppath.display()), 15);
    expected.insert(format!("{}/a/sample2", tmppath.display()), 24);
    expected.insert(format!("{}/a/sample3", tmppath.display()), 33);
    expected.insert(format!("{}/a/sample4", tmppath.display()), 42);
    expected.insert(format!("{}/a/sample5", tmppath.display()), 51);

    let pathname = tmp_dir.path().join("a/b/c/d");
    fs::create_dir_all(&pathname).unwrap();
    
    let subdirs = vec!["a", "a/b", "a/b/c", "a/b/c/d"];
    for subdir in subdirs {
        for i in 1..6 {
            let filename = tmp_dir.path().join(subdir).join(format!("sample{}", i));
            let mut tmpfile = fs::File::create(filename).unwrap();
            let mut contents = "Random strings".to_string();
            for _ in 1..i {
                contents = format!("{}{}", contents, "and more ".to_string());
            }
            writeln!(tmpfile, "{}", contents).unwrap();
        }
    }
    let _ = unix::fs::symlink(tmp_dir.path().join("sample1"), tmp_dir.path().join("skipped.txt"));

    let matches = App::new("DSintegration").get_matches();
    let disk_space = traverse(&tmp_dir.path().to_string_lossy().to_string(), &matches);

    tmp_dir.close().unwrap();
    assert_eq!(disk_space, expected);
}
