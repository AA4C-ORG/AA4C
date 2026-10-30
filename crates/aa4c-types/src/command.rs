use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 能力级别。静态属于命令，不随调用对象变化（ADR 0011）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Read,
    Act,
    Trust,
}

/// 可机器读取的副作用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    None,
    LocalWrite,
    Network,
    Trust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathAccess {
    None,
    Read,
    Write,
    ReadWrite,
}

/// 命令描述：`aa4c commands --json` 与 MCP 工具清单的唯一来源。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandDescriptor {
    pub id: String,
    pub summary: String,
    /// 参数 JSON Schema。
    pub params: Value,
    /// 结果 JSON Schema。
    pub result: Value,
    pub tier: Tier,
    pub effects: Vec<Effect>,
    pub path_access: PathAccess,
    pub long_running: bool,
    pub human_step: bool,
    pub idempotent: bool,
}
