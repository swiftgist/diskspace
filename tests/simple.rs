extern crate clap;
extern crate ds;
extern crate tempdir;
#[cfg(target_os = "linux")]
use clap::App;
#[cfg(target_os = "linux")]
use ds::traverse;
#[cfg(target_os = "linux")]
use std::collections::BTreeMap;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::unix;
use std::process::Command;
#[cfg(target_os = "linux")]
use tempdir::TempDir;

#[cfg(test)]
fn setup() {}

#[cfg(target_os = "linux")]
#[test]
fn simple() {
    setup();
    if let Ok(output) = Command::new("target/debug/ds").output() {
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains(" ./target/debug",));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn simple_verbose() {
    setup();
    if let Ok(output) = Command::new("target/debug/ds").arg(".").output() {
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains(" ./target/debug",));
    }
}

#[test]
fn simple_help() {
    setup();
    if let Ok(output) = Command::new("target/debug/ds").arg("-h").output() {
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Displays disk space usage",));
    }
}

#[cfg(target_os = "linux")]
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
    #[cfg(target_os = "linux")]
    let _ = unix::fs::symlink(
        tmp_dir.path().join("sample1"),
        tmp_dir.path().join("skipped.txt"),
    );

    let matches = App::new("DSintegration").get_matches();
    let disk_space = traverse(
        &vec![tmp_dir.path().to_string_lossy().to_string()],
        &matches,
    );

    tmp_dir.close().unwrap();
    assert_eq!(disk_space, expected);
}

#[cfg(target_os = "linux")]
#[test]
fn sample_permission_denied() {
    let mut expected = BTreeMap::new();

    // create directory structure and files
    let tmp_dir = TempDir::new("/tmp/dstest").unwrap();
    let tmppath = tmp_dir.path().to_owned();

    expected.insert(format!("{}/a", tmppath.display()), 165);
    expected.insert(format!("{}/a/sample1", tmppath.display()), 15);
    expected.insert(format!("{}/a/sample2", tmppath.display()), 24);
    expected.insert(format!("{}/a/sample3", tmppath.display()), 33);
    expected.insert(format!("{}/a/sample4", tmppath.display()), 42);
    expected.insert(format!("{}/a/sample5", tmppath.display()), 51);

    let pathname = tmp_dir.path().join("a/b/c/d");
    fs::create_dir_all(&pathname).unwrap();

    // let subdirs = vec!["a", "a/b", "a/b/c", "a/b/c/d"];
    let subdirs = vec!["a", "a/b"];
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

    let path = format!("{}/a/b", tmp_dir.path().display());
    dbg!(&path);
    let _ = Command::new("chmod").arg("0").arg(path).output();

    let matches = App::new("DSintegration").get_matches();
    let disk_space = traverse(
        &vec![tmp_dir.path().to_string_lossy().to_string()],
        &matches,
    );

    // tmp_dir.close().unwrap();
    assert_eq!(disk_space, expected);
}
