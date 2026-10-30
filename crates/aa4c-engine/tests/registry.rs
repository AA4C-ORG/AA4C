#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::time::Duration;

use aa4c_engine::{CallerContext, Command, Engine, Session};
use aa4c_types::{CommandDescriptor, CommandError, Effect, ErrorCode, PathAccess, Tier};
use serde_json::{Value, json};

fn descriptor(id: &str, tier: Tier, human_step: bool) -> CommandDescriptor {
    CommandDescriptor {
        id: id.to_string(),
        summary: "test command".to_string(),
        params: json!({"type": "object"}),
        result: json!({"type": "object"}),
        tier,
        effects: vec![Effect::None],
        path_access: PathAccess::None,
        long_running: false,
        human_step,
        idempotent: true,
    }
}

struct Probe {
    descriptor: CommandDescriptor,
    action: fn(&Session) -> Result<Value, CommandError>,
}

impl Command for Probe {
    fn descriptor(&self) -> &CommandDescriptor {
        &self.descriptor
    }

    fn run(&self, session: &Session, _: &CallerContext, _: &Value) -> Result<Value, CommandError> {
        (self.action)(session)
    }
}

fn probe(
    id: &str,
    tier: Tier,
    human_step: bool,
    action: fn(&Session) -> Result<Value, CommandError>,
) -> Box<Probe> {
    Box::new(Probe {
        descriptor: descriptor(id, tier, human_step),
        action,
    })
}

fn code(result: Result<Value, CommandError>) -> ErrorCode {
    result.expect_err("expected an error").code
}

#[test]
fn builtin_commands_are_listed_sorted_with_full_descriptions() {
    let engine = Engine::new();
    let list = engine.descriptors();
    let ids: Vec<&str> = list.iter().map(|d| d.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);
    for id in ["status.show", "send.files", "send.text", "devices.revoke"] {
        assert!(ids.contains(&id), "missing {id}");
    }
    let json = serde_json::to_value(&list).unwrap();
    let first = &json[0];
    for field in [
        "id",
        "summary",
        "params",
        "result",
        "tier",
        "effects",
        "path_access",
        "long_running",
        "human_step",
        "idempotent",
    ] {
        assert!(first.get(field).is_some(), "descriptor lacks {field}");
    }
}

#[test]
fn mcp_default_does_not_see_trust_commands() {
    let engine = Engine::new();
    let visible = engine.visible_descriptors(&CallerContext::mcp());
    assert!(visible.iter().all(|d| d.tier <= Tier::Act));
    assert!(visible.iter().any(|d| d.id == "send.files"));
    assert!(!visible.iter().any(|d| d.id == "devices.revoke"));
}

#[test]
fn unknown_command_is_not_found() {
    let engine = Engine::new();
    assert_eq!(
        code(engine.run(&CallerContext::test(), "no.such", json!({}))),
        ErrorCode::NotFound
    );
}

#[test]
fn status_show_runs_for_everyone() {
    let engine = Engine::new();
    for ctx in [
        CallerContext::mcp(),
        CallerContext::cli(),
        CallerContext::ui(false),
    ] {
        let result = engine.run(&ctx, "status.show", json!({})).unwrap();
        assert!(result.get("version").and_then(Value::as_str).is_some());
    }
}

#[test]
fn trust_command_is_denied_to_default_mcp_even_if_params_claim_otherwise() {
    let engine = Engine::new();
    let ctx = CallerContext::mcp();
    let params = json!({"device": "abc"});
    assert_eq!(
        code(engine.run(&ctx, "devices.revoke", params)),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn trust_command_needs_a_human_even_with_allow_trust() {
    let engine = Engine::new();
    let mcp = CallerContext::mcp().allow_trust();
    assert_eq!(
        code(engine.run(&mcp, "devices.revoke", json!({"device": "abc"}))),
        ErrorCode::NeedsHuman
    );
    // Without a person the UI cannot reach trust at all.
    let ui_without_human = CallerContext::ui(false);
    assert_eq!(
        code(engine.run(
            &ui_without_human,
            "devices.revoke",
            json!({"device": "abc"})
        )),
        ErrorCode::PermissionDenied
    );
}

#[test]
fn human_at_the_ui_reaches_the_trust_command() {
    let engine = Engine::new();
    let ctx = CallerContext::ui(true);
    // Policy passes; the stub implementation reports it is not built yet.
    assert_eq!(
        code(engine.run(&ctx, "devices.revoke", json!({"device": "abc"}))),
        ErrorCode::Unsupported
    );
}

#[test]
fn params_cannot_forge_the_caller_context() {
    let engine = Engine::new();
    for key in [
        "origin",
        "max_tier",
        "human_present",
        "read_roots",
        "write_roots",
        "allowed_targets",
        "caller",
    ] {
        let params = json!({"text": "hi", "to": "abc", key: "ui"});
        assert_eq!(
            code(engine.run(&CallerContext::test(), "send.text", params)),
            ErrorCode::InvalidParams,
            "{key} must be rejected"
        );
    }
}

#[test]
fn a_panicking_command_becomes_an_internal_error_and_the_engine_survives() {
    let mut engine = Engine::empty();
    engine.register(probe("test.boom", Tier::Read, false, |_| panic!("boom")));
    engine.register(probe("test.ok", Tier::Read, false, |_| {
        Ok(json!({"ok": true}))
    }));
    let ctx = CallerContext::test();
    assert_eq!(
        code(engine.run(&ctx, "test.boom", json!({}))),
        ErrorCode::Internal
    );
    assert_eq!(
        engine.run(&ctx, "test.ok", json!({})).unwrap(),
        json!({"ok": true})
    );
}

#[test]
fn commands_publish_events_to_subscribers() {
    let mut engine = Engine::empty();
    engine.register(probe("test.emit", Tier::Read, false, |session| {
        session.emit("transfer.progress", json!({"done": 1}));
        Ok(json!({}))
    }));
    let events = engine.subscribe();
    let dropped = engine.subscribe();
    drop(dropped);

    engine
        .run(&CallerContext::test(), "test.emit", json!({}))
        .unwrap();

    let event = events.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(event.kind, "transfer.progress");
    assert_eq!(event.data, json!({"done": 1}));
    assert!(event.at > 0);
}
