//! `send.*`：先只做授权，真实发送在 M1。授权通过后返回 `unsupported`。

use aa4c_types::{CommandDescriptor, CommandError, Effect, PathAccess, Tier};
use serde::Deserialize;
use serde_json::{Value, json};

use super::describe;
use crate::context::CallerContext;
use crate::engine::{Command, Engine, Session};
use crate::policy;

/// 单次请求最多带多少个路径。
const MAX_PATHS: usize = 1000;

fn check_target(ctx: &CallerContext, to: &str) -> Result<(), CommandError> {
    if ctx.allowed_targets.allows(&to.to_string()) {
        Ok(())
    } else {
        Err(
            CommandError::permission_denied("target was not granted to this session")
                .with_hint("ask the user to grant it with --allow-to"),
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SendFiles {
    paths: Vec<String>,
    to: String,
}

struct SendFilesCommand(CommandDescriptor);

impl SendFilesCommand {
    fn parse(params: &Value) -> Result<SendFiles, CommandError> {
        let parsed = SendFiles::deserialize(params)
            .map_err(|e| CommandError::invalid_params(e.to_string()))?;
        if parsed.paths.is_empty() || parsed.paths.len() > MAX_PATHS {
            return Err(CommandError::invalid_params(format!(
                "paths must hold 1 to {MAX_PATHS} entries"
            )));
        }
        Ok(parsed)
    }
}

impl Command for SendFilesCommand {
    fn descriptor(&self) -> &CommandDescriptor {
        &self.0
    }

    fn authorize(&self, ctx: &CallerContext, params: &Value) -> Result<(), CommandError> {
        let parsed = Self::parse(params)?;
        check_target(ctx, &parsed.to)?;
        let mut total: u64 = 0;
        for path in &parsed.paths {
            let meta = policy::check_read(ctx, path)?;
            if !meta.is_file() {
                // Directories and links need recursive accounting; not before M1.
                return Err(CommandError::unsupported(
                    "only regular files are supported so far",
                ));
            }
            total = total.saturating_add(meta.len());
        }
        if let Some(budget) = ctx.max_send_bytes
            && total > budget
        {
            return Err(
                CommandError::permission_denied("request exceeds the byte budget")
                    .with_hint("ask the user to raise it with --max-send-bytes"),
            );
        }
        Ok(())
    }

    fn run(&self, _: &Session, _: &CallerContext, _: &Value) -> Result<Value, CommandError> {
        Err(CommandError::unsupported(
            "send.files is not implemented yet",
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SendText {
    #[allow(dead_code)]
    text: String,
    to: String,
}

struct SendTextCommand(CommandDescriptor);

impl Command for SendTextCommand {
    fn descriptor(&self) -> &CommandDescriptor {
        &self.0
    }

    fn authorize(&self, ctx: &CallerContext, params: &Value) -> Result<(), CommandError> {
        let parsed = SendText::deserialize(params)
            .map_err(|e| CommandError::invalid_params(e.to_string()))?;
        check_target(ctx, &parsed.to)
    }

    fn run(&self, _: &Session, _: &CallerContext, _: &Value) -> Result<Value, CommandError> {
        Err(CommandError::unsupported(
            "send.text is not implemented yet",
        ))
    }
}

pub(super) fn register(engine: &mut Engine) {
    let mut files = describe(
        "send.files",
        "Send files to one of my devices or a contact.",
        Tier::Act,
        json!({
            "type": "object",
            "required": ["paths", "to"],
            "additionalProperties": false,
            "properties": {
                "paths": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": MAX_PATHS},
                "to": {"type": "string"}
            }
        }),
        json!({"type": "object", "required": ["transfer"], "properties": {"transfer": {"type": "string"}}}),
    );
    files.effects = vec![Effect::Network];
    files.path_access = PathAccess::Read;
    files.long_running = true;
    files.idempotent = false;
    engine.register(Box::new(SendFilesCommand(files)));

    let mut text = describe(
        "send.text",
        "Send a piece of text to one of my devices or a contact.",
        Tier::Act,
        json!({
            "type": "object",
            "required": ["text", "to"],
            "additionalProperties": false,
            "properties": {"text": {"type": "string"}, "to": {"type": "string"}}
        }),
        json!({"type": "object", "required": ["transfer"], "properties": {"transfer": {"type": "string"}}}),
    );
    text.effects = vec![Effect::Network];
    text.long_running = true;
    text.idempotent = false;
    engine.register(Box::new(SendTextCommand(text)));
}
