use std::process::Command;

#[test]
fn prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_weft"))
        .arg("--help")
        .output()
        .expect("the Weft CLI should start");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}
