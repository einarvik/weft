use std::{fs, process::Command};

#[test]
fn prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_weft"))
        .arg("--help")
        .output()
        .expect("the Weft CLI should start");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}

#[test]
fn writes_native_site_artifacts() {
    let temporary = std::env::temp_dir().join(format!("weft-cli-test-{}", std::process::id()));
    let source = temporary.join("site.wft");
    let output = temporary.join("dist");
    fs::create_dir_all(&temporary).expect("temporary directory should exist");
    fs::write(
        &source,
        "site Acme\npage /:\n  hero:\n    title \"Native web\"\n",
    )
    .expect("fixture should be written");

    let result = Command::new(env!("CARGO_BIN_EXE_weft"))
        .args([source.as_os_str(), "--output".as_ref(), output.as_os_str()])
        .output()
        .expect("the Weft CLI should start");

    assert!(result.status.success());
    assert!(
        fs::read_to_string(output.join("index.html"))
            .expect("HTML output")
            .contains("<h1 class=\"weft-title\">Native web</h1>")
    );
    assert!(
        fs::read_to_string(output.join("site.css"))
            .expect("CSS output")
            .contains(":root")
    );

    fs::remove_dir_all(temporary).expect("temporary artifacts should be removable");
}

#[test]
fn checks_source_without_writing_artifacts() {
    let temporary = std::env::temp_dir().join(format!("weft-check-test-{}", std::process::id()));
    let source = temporary.join("site.wft");
    fs::create_dir_all(&temporary).expect("temporary directory should exist");
    fs::write(
        &source,
        "site Acme\npage /:\n  hero:\n    title \"Native web\"\n",
    )
    .expect("fixture should be written");

    let result = Command::new(env!("CARGO_BIN_EXE_weft"))
        .args(["check".as_ref(), source.as_os_str()])
        .output()
        .expect("the Weft CLI should start");

    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("Checked"));
    assert!(!temporary.join("dist").exists());

    fs::remove_dir_all(temporary).expect("temporary artifacts should be removable");
}

#[test]
fn check_preserves_line_aware_compiler_errors() {
    let temporary = std::env::temp_dir().join(format!("weft-check-error-{}", std::process::id()));
    let source = temporary.join("broken.wft");
    fs::create_dir_all(&temporary).expect("temporary directory should exist");
    fs::write(&source, "site Acme\npage /:\n  title \"Invalid nesting\"\n")
        .expect("fixture should be written");

    let result = Command::new(env!("CARGO_BIN_EXE_weft"))
        .args(["check".as_ref(), source.as_os_str()])
        .output()
        .expect("the Weft CLI should start");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("line 3:"));
    assert!(!temporary.join("dist").exists());

    fs::remove_dir_all(temporary).expect("temporary artifacts should be removable");
}
