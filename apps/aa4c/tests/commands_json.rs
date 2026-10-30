#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::process::Command;

fn aa4c(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aa4c"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn commands_json_lists_every_command_description() {
    let output = aa4c(&["commands", "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let status = list
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "status.show")
        .expect("status.show is listed");
    assert_eq!(status["tier"], "read");
    assert!(status["params"].is_object());
}

#[test]
fn commands_without_json_prints_one_line_per_command() {
    let output = aa4c(&["commands"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.lines().any(|l| l.starts_with("status.show")));
}

#[test]
fn unknown_subcommand_is_a_usage_error() {
    let output = aa4c(&["frobnicate"]);
    assert_eq!(output.status.code(), Some(2));
}
