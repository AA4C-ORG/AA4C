use aa4c_types::{CommandDescriptor, CommandError, Tier};
use serde_json::{Value, json};

use super::{describe, empty_object_schema};
use crate::context::CallerContext;
use crate::engine::{Command, Engine, Session};

struct StatusShow(CommandDescriptor);

impl Command for StatusShow {
    fn descriptor(&self) -> &CommandDescriptor {
        &self.0
    }

    fn run(&self, _: &Session, _: &CallerContext, _: &Value) -> Result<Value, CommandError> {
        Ok(json!({"version": env!("CARGO_PKG_VERSION")}))
    }
}

pub(super) fn register(engine: &mut Engine) {
    engine.register(Box::new(StatusShow(describe(
        "status.show",
        "Show the local engine status.",
        Tier::Read,
        empty_object_schema(),
        json!({"type": "object", "required": ["version"], "properties": {"version": {"type": "string"}}}),
    ))));
}
