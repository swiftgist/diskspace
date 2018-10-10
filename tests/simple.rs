
use std::process::Command;

#[cfg(test)]
fn setup() {}

#[test]
fn simple() {
    setup();
    let output = Command::new("target/debug/ds").output().expect(
        "failed to execute ds",
    );

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "M ./target/debug/ds",
    ));
}
