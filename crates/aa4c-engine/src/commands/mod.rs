//! 内置命令，每组一个模块。

mod devices;
mod send;
mod status;

use aa4c_types::{CommandDescriptor, Effect, PathAccess, Tier};
use serde_json::{Value, json};

use crate::engine::Engine;

pub(crate) fn register_all(engine: &mut Engine) {
    status::register(engine);
    send::register(engine);
    devices::register(engine);
}

/// 描述的公共起点：无副作用、无路径、短任务。
fn describe(
    id: &str,
    summary: &str,
    tier: Tier,
    params: Value,
    result: Value,
) -> CommandDescriptor {
    CommandDescriptor {
        id: id.to_string(),
        summary: summary.to_string(),
        params,
        result,
        tier,
        effects: vec![Effect::None],
        path_access: PathAccess::None,
        long_running: false,
        human_step: false,
        idempotent: true,
    }
}

fn empty_object_schema() -> Value {
    json!({"type": "object", "additionalProperties": false})
}
