//! End-to-end tests for the `ekips` binary.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn ekips(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ekips"))
        .args(args)
        // Make the tests independent of the machine's language.
        .env_remove("EKIPS_LANG")
        .env("LANG", "en_US.UTF-8")
        .output()
        .expect("failed to start ekips")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn valid_program_passes() {
    let path = fixture("square.ekips");
    let out = ekips(&["--lang", "en", "check", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("OK"));
}

#[test]
fn typo_fails_with_english_hint() {
    let path = fixture("typo.ekips");
    let out = ekips(&["--lang", "en", "check", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    let err = text(&out.stderr);
    assert!(
        err.contains("error: I don't understand the word `cosnt` here."),
        "{err}"
    );
    assert!(err.contains("help: Did you mean `const`?"), "{err}");
    assert!(err.contains("typo.ekips:2:1"), "{err}");
    // Not a terminal, so no color codes.
    assert!(!err.contains('\u{1b}'), "{err}");
}

#[test]
fn typo_fails_with_german_hint() {
    let path = fixture("typo.ekips");
    let out = ekips(&["check", "--lang", "de", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    let err = text(&out.stderr);
    assert!(
        err.contains("Fehler: Das Wort `cosnt` verstehe ich hier nicht."),
        "{err}"
    );
    assert!(err.contains("Hilfe: Meintest du `const`?"), "{err}");
}

#[test]
fn color_always_adds_ansi_codes() {
    let path = fixture("typo.ekips");
    let out = ekips(&["--color", "always", "check", path.to_str().unwrap()]);
    assert!(text(&out.stderr).contains('\u{1b}'));
}

#[test]
fn lang_env_variable_is_used() {
    let path = fixture("typo.ekips");
    let out = Command::new(env!("CARGO_BIN_EXE_ekips"))
        .args(["check", path.to_str().unwrap()])
        .env("EKIPS_LANG", "de")
        .output()
        .unwrap();
    assert!(text(&out.stderr).contains("Meintest du"));
}

#[test]
fn folder_is_checked_recursively() {
    let dir = fixture("");
    let out = ekips(&["--lang", "en", "check", dir.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stdout).contains("Checked 2 files: 1 error, 0 warnings."));
}

#[test]
fn missing_file_is_a_usage_error() {
    let out = ekips(&["--lang", "en", "check", "does-not-exist.ekips"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("does not exist"));
}

#[test]
fn ast_is_printed() {
    let path = fixture("square.ekips");
    let out = ekips(&["check", "--ast", path.to_str().unwrap()]);
    assert!(text(&out.stdout).contains("Program {"));
}

#[test]
fn planned_commands_explain_themselves() {
    for args in [
        &["run", "--sim", "x.ekips"][..],
        &["run", "x.ekips"],
        &["upload", "x.ekips"],
        &["monitor"],
        &["flash-firmware"],
        &["backup-firmware"],
    ] {
        let mut all = vec!["--lang", "de"];
        all.extend_from_slice(args);
        let out = ekips(&all);
        assert_eq!(out.status.code(), Some(3), "{args:?}");
        assert!(text(&out.stderr).contains("gibt es noch nicht"), "{args:?}");
    }
}

#[test]
fn help_is_translated() {
    let out = ekips(&["--lang", "de", "--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = text(&out.stdout);
    assert!(help.contains("Aufruf:"), "{help}");
    assert!(help.contains("Befehle:"), "{help}");
    assert!(help.contains("Programme auf Fehler prüfen"), "{help}");
}
