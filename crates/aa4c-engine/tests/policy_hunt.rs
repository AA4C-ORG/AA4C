//! MCP 的目标、路径、字节预算越权测试，先于真实发送存在（路线图 M0 第 4 条）。
//! 授权通过的标志是命令走到「尚未实现」的桩（`unsupported`）；被拒的是 `permission_denied` 等。
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::Path;

use aa4c_engine::{CallerContext, Engine};
use aa4c_types::ErrorCode;
use serde_json::{Value, json};

fn send(
    engine: &Engine,
    ctx: &CallerContext,
    paths: &[&Path],
    to: &str,
) -> Result<Value, aa4c_types::CommandError> {
    let paths: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    engine.run(ctx, "send.files", json!({"paths": paths, "to": to}))
}

fn code(result: Result<Value, aa4c_types::CommandError>) -> ErrorCode {
    result.expect_err("expected an error").code
}

struct Fixture {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
    outside: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("docs");
    let secret_dir = dir.path().join("secret");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&secret_dir).unwrap();
    fs::write(root.join("report.txt"), vec![0u8; 100]).unwrap();
    fs::write(secret_dir.join("key.txt"), b"secret").unwrap();
    Fixture {
        root,
        outside: secret_dir.join("key.txt"),
        _dir: dir,
    }
}

fn mcp(f: &Fixture) -> CallerContext {
    CallerContext::mcp()
        .with_read_root(&f.root)
        .with_allowed_target("phone")
        .with_max_send_bytes(1000)
}

#[test]
fn a_file_under_the_root_to_an_allowed_target_passes_authorization() {
    let f = fixture();
    let result = send(
        &Engine::new(),
        &mcp(&f),
        &[&f.root.join("report.txt")],
        "phone",
    );
    assert_eq!(code(result), ErrorCode::Unsupported);
}

#[test]
fn a_file_outside_every_root_is_denied() {
    let f = fixture();
    assert_eq!(
        code(send(&Engine::new(), &mcp(&f), &[&f.outside], "phone")),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn one_bad_path_denies_the_whole_request() {
    let f = fixture();
    let paths = [f.root.join("report.txt"), f.outside.clone()];
    let refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    assert_eq!(
        code(send(&Engine::new(), &mcp(&f), &refs, "phone")),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn parent_directory_components_are_denied() {
    let f = fixture();
    let sneaky = f.root.join("..").join("secret").join("key.txt");
    assert_eq!(
        code(send(&Engine::new(), &mcp(&f), &[&sneaky], "phone")),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn relative_paths_are_denied() {
    let f = fixture();
    assert_eq!(
        code(send(
            &Engine::new(),
            &mcp(&f),
            &[Path::new("report.txt")],
            "phone"
        )),
        ErrorCode::PermissionDenied
    );
}

#[cfg(unix)]
#[test]
fn a_symlink_under_the_root_pointing_outside_is_denied() {
    let f = fixture();
    let link = f.root.join("innocent.txt");
    std::os::unix::fs::symlink(&f.outside, &link).unwrap();
    assert_eq!(
        code(send(&Engine::new(), &mcp(&f), &[&link], "phone")),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn no_granted_root_means_no_access() {
    let f = fixture();
    let ctx = CallerContext::mcp()
        .with_allowed_target("phone")
        .with_max_send_bytes(1000);
    assert_eq!(
        code(send(
            &Engine::new(),
            &ctx,
            &[&f.root.join("report.txt")],
            "phone"
        )),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn a_target_that_was_not_granted_is_denied() {
    let f = fixture();
    let result = send(
        &Engine::new(),
        &mcp(&f),
        &[&f.root.join("report.txt")],
        "stranger",
    );
    assert_eq!(code(result), ErrorCode::PermissionDenied);
}

#[test]
fn a_request_over_the_byte_budget_is_denied() {
    let f = fixture();
    let ctx = mcp(&f).with_max_send_bytes(99);
    assert_eq!(
        code(send(
            &Engine::new(),
            &ctx,
            &[&f.root.join("report.txt")],
            "phone"
        )),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn a_missing_file_is_not_found() {
    let f = fixture();
    assert_eq!(
        code(send(
            &Engine::new(),
            &mcp(&f),
            &[&f.root.join("nope.txt")],
            "phone"
        )),
        ErrorCode::NotFound
    );
}

#[test]
fn text_to_an_ungranted_target_is_denied() {
    let f = fixture();
    let result = Engine::new().run(
        &mcp(&f),
        "send.text",
        json!({"text": "hi", "to": "stranger"}),
    );
    assert_eq!(code(result), ErrorCode::PermissionDenied);
}

#[test]
fn the_cli_inherits_user_permissions() {
    let f = fixture();
    let result = send(
        &Engine::new(),
        &CallerContext::cli(),
        &[&f.outside],
        "anyone",
    );
    assert_eq!(code(result), ErrorCode::Unsupported);
}
