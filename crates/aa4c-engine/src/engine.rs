use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::Receiver;

use aa4c_types::{CommandDescriptor, CommandError, Event};
use serde_json::Value;

use crate::context::CallerContext;
use crate::events::EventBus;

/// 命令实现可用的引擎状态（身份、状态库、网络等随各 crate 落地后接入）。
#[derive(Debug, Default)]
pub struct Session {
    events: EventBus,
}

impl Session {
    pub fn emit(&self, kind: &str, data: Value) {
        self.events.publish(kind, data);
    }
}

/// 注册表里的一个命令。
pub trait Command: Send + Sync {
    fn descriptor(&self) -> &CommandDescriptor;

    /// 能力级别与人类步骤之外的授权（根目录、目标、预算）。通过后才会调用 `run`。
    fn authorize(&self, _ctx: &CallerContext, _params: &Value) -> Result<(), CommandError> {
        Ok(())
    }

    fn run(
        &self,
        session: &Session,
        ctx: &CallerContext,
        params: &Value,
    ) -> Result<Value, CommandError>;
}

/// `CallerContext` 的字段名；出现在参数里一律拒绝。
const RESERVED_PARAM_KEYS: &[&str] = &[
    "caller",
    "origin",
    "max_tier",
    "human_present",
    "read_roots",
    "write_roots",
    "allowed_targets",
    "max_send_bytes",
    "request_id",
];

#[derive(Default)]
pub struct Engine {
    commands: BTreeMap<String, Box<dyn Command>>,
    session: Session,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("commands", &self.commands.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Engine {
    /// 带全部内置命令的引擎。
    pub fn new() -> Self {
        let mut engine = Self::empty();
        crate::commands::register_all(&mut engine);
        engine
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands
            .insert(command.descriptor().id.clone(), command);
    }

    /// 全部命令描述，按 id 排序。
    pub fn descriptors(&self) -> Vec<CommandDescriptor> {
        self.commands
            .values()
            .map(|c| c.descriptor().clone())
            .collect()
    }

    /// 该调用者能看到（也可能调用）的命令：能力级别不超过其上限。MCP 工具清单据此生成。
    pub fn visible_descriptors(&self, ctx: &CallerContext) -> Vec<CommandDescriptor> {
        self.descriptors()
            .into_iter()
            .filter(|d| d.tier <= ctx.max_tier)
            .collect()
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.session.events.subscribe()
    }

    /// 唯一的命令入口。顺序：查找 → 拒绝伪造上下文 → 能力级别 → 人类步骤 → 命令授权 → 执行（兜底捕获 panic）。
    pub fn run(&self, ctx: &CallerContext, id: &str, params: Value) -> Result<Value, CommandError> {
        let command = self
            .commands
            .get(id)
            .ok_or_else(|| CommandError::not_found(format!("unknown command: {id}")))?;
        let descriptor = command.descriptor();

        if let Some(object) = params.as_object()
            && let Some(key) = RESERVED_PARAM_KEYS
                .iter()
                .find(|k| object.contains_key(**k))
        {
            return Err(CommandError::invalid_params(format!(
                "parameter `{key}` is reserved"
            )));
        }
        if descriptor.tier > ctx.max_tier {
            return Err(CommandError::permission_denied(format!(
                "{id} needs a higher capability level"
            ))
            .with_hint("trust-level changes are made by a person in the app"));
        }
        if descriptor.human_step && !ctx.human_present {
            return Err(CommandError::needs_human(format!(
                "{id} must be completed by a person"
            )));
        }

        command.authorize(ctx, &params)?;
        catch_unwind(AssertUnwindSafe(|| {
            command.run(&self.session, ctx, &params)
        }))
        .unwrap_or_else(|_| Err(CommandError::internal(format!("{id} failed unexpectedly"))))
    }
}
