//! Session、命令注册表、CallerContext 与事件流。
//!
//! 所有外壳（CLI、MCP、桌面、手机、插件）只通过 [`Engine::run`] 调用命令；测试也打在这里。

mod commands;
mod context;
mod engine;
mod events;
mod policy;

pub use context::{CallerContext, Origin, Scope};
pub use engine::{Command, Engine, Session};
