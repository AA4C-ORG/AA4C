use aa4c_types::{CommandDescriptor, CommandError, Effect, Tier};
use serde::Deserialize;
use serde_json::{Value, json};

use super::describe;
use crate::context::CallerContext;
use crate::engine::{Command, Engine, Session};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevokeParams {
    #[allow(dead_code)]
    device: String,
}

struct DevicesRevoke(CommandDescriptor);

impl Command for DevicesRevoke {
    fn descriptor(&self) -> &CommandDescriptor {
        &self.0
    }

    fn run(&self, _: &Session, _: &CallerContext, params: &Value) -> Result<Value, CommandError> {
        RevokeParams::deserialize(params)
            .map_err(|e| CommandError::invalid_params(e.to_string()))?;
        Err(CommandError::unsupported(
            "devices.revoke is not implemented yet",
        ))
    }
}

pub(super) fn register(engine: &mut Engine) {
    let mut descriptor = describe(
        "devices.revoke",
        "Revoke one of my devices.",
        Tier::Trust,
        json!({"type": "object", "required": ["device"], "additionalProperties": false, "properties": {"device": {"type": "string"}}}),
        json!({"type": "object"}),
    );
    descriptor.effects = vec![Effect::Trust, Effect::LocalWrite, Effect::Network];
    descriptor.human_step = true;
    descriptor.idempotent = true;
    engine.register(Box::new(DevicesRevoke(descriptor)));
}
