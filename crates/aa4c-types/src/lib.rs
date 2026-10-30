//! 共享数据类型：指纹、档位、错误码、命令描述。

mod command;
mod error;
mod event;

pub use command::{CommandDescriptor, Effect, PathAccess, Tier};
pub use error::{CommandError, ErrorCode};
pub use event::Event;
