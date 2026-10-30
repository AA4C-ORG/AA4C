//! 对每个命令喂对抗性参数：必须返回结果或结构化错误，不能 panic（`internal` 即兜底捕获到了 panic）。
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use aa4c_engine::{CallerContext, Engine};
use aa4c_types::ErrorCode;
use serde_json::{Value, json};

fn hostile_params() -> Vec<Value> {
    let long = "a".repeat(1 << 20);
    vec![
        json!(null),
        json!(true),
        json!(0),
        json!(-1),
        json!(1.5e308),
        json!("string"),
        json!([]),
        json!({}),
        json!({"paths": null, "to": null, "text": null, "device": null}),
        json!({"paths": "not-a-list", "to": 7, "text": [], "device": {}}),
        json!({"paths": [], "to": "x"}),
        json!({"paths": [null, 1, {}], "to": "x"}),
        json!({"paths": [long.clone()], "to": long.clone(), "text": long.clone(), "device": long}),
        json!({"paths": ["\u{0}", "\u{202e}evil", "e\u{301}\u{301}\u{301}", "\u{d7ff}"], "to": "\u{0}", "text": "\u{0}\u{1f600}"}),
        json!({"paths": ["/"], "to": ""}),
        json!({"paths": ["../../etc/passwd"], "to": "x"}),
        json!({"unknown_field": 1}),
    ]
}

#[test]
fn no_command_panics_on_hostile_params() {
    let engine = Engine::new();
    let contexts = [
        CallerContext::mcp().allow_trust(),
        CallerContext::mcp(),
        CallerContext::cli(),
        CallerContext::ui(true),
        CallerContext::test(),
    ];
    for descriptor in engine.descriptors() {
        for ctx in &contexts {
            for params in hostile_params() {
                if let Err(error) = engine.run(ctx, &descriptor.id, params.clone()) {
                    assert_ne!(
                        error.code,
                        ErrorCode::Internal,
                        "{} panicked on {params}",
                        descriptor.id
                    );
                }
            }
        }
    }
}
