//! # Output tests
//!
//! What the binary prints and how it exits, under `--json` and in text, run
//! over the clean fixture.

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

const CLEAN: &str = "tests/data/clean.vcf";

/// Runs the binary with no editor in its environment, `stdin` piped in.
fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tcard"))
        .args(args)
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tcard");

    let mut pipe = child.stdin.take().expect("stdin");
    pipe.write_all(stdin.as_bytes()).expect("write stdin");
    drop(pipe);

    child.wait_with_output().expect("run tcard")
}

/// The one JSON value the binary printed, and its exit code.
fn json(args: &[&str]) -> (Value, i32) {
    let output = run(args, "");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let value = serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("{err}: {stdout}"));
    (value, output.status.code().expect("exit code"))
}

#[test]
fn a_template_prints_its_contents_under_json() {
    let (value, code) = json(&["--json", "template", CLEAN]);
    let toml = String::from_utf8(run(&["template", CLEAN], "").stdout).unwrap();

    assert_eq!(code, 0);
    assert_eq!(value["contents"].as_str(), Some(toml.as_str()));
}

#[test]
fn a_written_template_prints_its_receipt_under_json_and_nothing_in_text() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("form.toml");
    let target = path.to_str().unwrap();

    let (value, code) = json(&["--json", "template", CLEAN, "-o", target]);
    let size = fs::metadata(&path).unwrap().len();

    assert_eq!(code, 0);
    assert_eq!(value["path"].as_str(), Some(target));
    assert_eq!(value["size"].as_u64(), Some(size));

    let text = run(&["template", CLEAN, "-o", target], "");
    assert!(text.status.success());
    assert!(text.stdout.is_empty());
}

#[test]
fn an_applied_form_prints_the_card_under_json() {
    let form = String::from_utf8(run(&["template", CLEAN], "").stdout).unwrap();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("form.toml");
    fs::write(&path, form).unwrap();
    let card = fs::read_to_string(CLEAN).unwrap();

    let (value, code) = json(&["--json", "apply", path.to_str().unwrap(), &card]);

    assert_eq!(code, 0);
    assert_eq!(value["contents"].as_str(), Some(card.as_str()));
}

#[test]
fn a_broken_form_is_coded_template_unreadable() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("form.toml");
    fs::write(&path, "name = \n").unwrap();
    let card = fs::read_to_string(CLEAN).unwrap();

    let (value, code) = json(&["--json", "apply", path.to_str().unwrap(), &card]);

    assert_eq!(code, 1);
    assert_eq!(value["code"], "template-unreadable");
}

#[test]
fn an_unreadable_card_is_coded_card_unreadable() {
    let (value, code) = json(&["--json", "template", "BEGIN:VCARD\r\n"]);

    assert_eq!(code, 1);
    assert_eq!(value["code"], "card-unreadable");
}

#[test]
fn an_edit_with_no_editor_is_coded_editor_missing() {
    let (value, code) = json(&["--json", "edit"]);

    assert_eq!(code, 1);
    assert_eq!(value["code"], "editor-missing");
}

#[test]
fn an_editor_exiting_with_a_failure_is_coded_editor_failed() {
    let (value, code) = json(&["--json", "edit", "--editor", "false"]);

    assert_eq!(code, 1);
    assert_eq!(value["code"], "editor-failed");

    // NOTE: the error names the buffer it kept, which is the recovery.
    let kept = value["error"].as_str().unwrap();
    let path = kept.split('"').nth(1).expect("kept path");
    fs::remove_file(Path::new(path)).unwrap();
}

#[test]
fn a_refused_command_line_exits_2_coded_usage_invalid() {
    let (value, code) = json(&["--json", "--frobnicate"]);

    assert_eq!(code, 2);
    assert_eq!(value["code"], "usage-invalid");
}

#[test]
fn two_stdin_inputs_are_coded_usage_invalid() {
    let (value, code) = json(&["--json", "apply", "-", "-"]);

    assert_eq!(code, 1);
    assert_eq!(value["code"], "usage-invalid");
}
