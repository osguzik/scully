use assert_cmd::Command;
use predicates::str::contains;

fn scully() -> Command {
    Command::cargo_bin("scully").unwrap()
}

#[test]
fn decodes_write() {
    scully()
        .args(["echo", "hi"])
        .assert()
        .stdout("hi\n")
        .stderr(contains(r#"write(1, "hi\n", 3) = 3"#));
}

#[test]
fn decodes_failed_open() {
    scully()
        .args(["cat", "/file"])
        .assert()
        .code(1)
        .stderr(contains(r#"openat(AT_FDCWD, "/file", O_RDONLY) = -1 ENOENT"#));
}

#[test]
fn prints_stats_table() {
    scully()
        .args(["-c", "true"])
        .assert()
        .stderr(contains("% time"));
}
