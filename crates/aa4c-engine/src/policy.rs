//! 授权检查：根目录、目标、字节预算。命令的 `authorize` 调用这里，不各自实现。

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use aa4c_types::CommandError;

use crate::context::{CallerContext, Scope};

/// 检查 `path` 可被读取，返回其元数据（不跟随符号链接）。
///
/// 受限上下文要求：绝对路径、不含 `..`、落在某个授权根之下，且根以下的每一级都不是符号链接。
/// 这是词法 + 逐级检查，仍有检查与使用之间的竞态；M0 选定 `cap-std` 或 `openat` 后改为相对根目录句柄打开
/// （见 book/src/surfaces/agents.md）。
pub(crate) fn check_read(ctx: &CallerContext, path: &str) -> Result<fs::Metadata, CommandError> {
    let path = Path::new(path);
    if let Scope::Only(roots) = &ctx.read_roots {
        check_under_roots(roots.iter(), path)?;
    }
    fs::symlink_metadata(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => CommandError::not_found("no such file"),
        io::ErrorKind::InvalidInput => CommandError::invalid_params("invalid path"),
        _ => CommandError::new(aa4c_types::ErrorCode::Io, "cannot read file"),
    })
}

fn denied() -> CommandError {
    CommandError::permission_denied("path is outside the granted read roots")
        .with_hint("ask the user to grant the directory with --read-root")
}

fn check_under_roots<'a>(
    roots: impl Iterator<Item = &'a PathBuf>,
    path: &Path,
) -> Result<(), CommandError> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(denied());
    }
    let Some((root, below)) = roots
        .filter_map(|root| path.strip_prefix(root).ok().map(|below| (root, below)))
        .next()
    else {
        return Err(denied());
    };
    let mut current = root.clone();
    for component in below.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(denied()),
            // A missing or unreadable component is reported by the caller as not_found / io.
            _ => {}
        }
    }
    Ok(())
}
