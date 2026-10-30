use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use aa4c_types::Tier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Ui,
    Cli,
    Mcp,
    Plugin,
    Device,
    Test,
}

/// 文件根与接收目标的授权范围。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope<T: Ord> {
    /// 继承系统用户权限（CLI、人在场的界面）。
    Unrestricted,
    /// 只限这些；空集合表示什么都不许。
    Only(BTreeSet<T>),
}

impl<T: Ord> Scope<T> {
    pub fn allows(&self, item: &T) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Only(set) => set.contains(item),
        }
    }
}

/// 由可信入口创建；命令参数里的同名字段一律拒绝，不会进入这里。
#[derive(Debug, Clone)]
pub struct CallerContext {
    pub origin: Origin,
    pub max_tier: Tier,
    pub human_present: bool,
    pub read_roots: Scope<PathBuf>,
    pub write_roots: Scope<PathBuf>,
    pub allowed_targets: Scope<String>,
    /// 单次发送的字节预算；`None` 表示不限。
    pub max_send_bytes: Option<u64>,
}

impl CallerContext {
    fn restricted(origin: Origin) -> Self {
        Self {
            origin,
            max_tier: Tier::Act,
            human_present: false,
            read_roots: Scope::Only(BTreeSet::new()),
            write_roots: Scope::Only(BTreeSet::new()),
            allowed_targets: Scope::Only(BTreeSet::new()),
            max_send_bytes: Some(0),
        }
    }

    fn unrestricted(origin: Origin, max_tier: Tier, human_present: bool) -> Self {
        Self {
            origin,
            max_tier,
            human_present,
            read_roots: Scope::Unrestricted,
            write_roots: Scope::Unrestricted,
            allowed_targets: Scope::Unrestricted,
            max_send_bytes: None,
        }
    }

    /// MCP 会话的默认上下文：最高 `act`，没有任何根目录、目标与字节预算。
    pub fn mcp() -> Self {
        Self::restricted(Origin::Mcp)
    }

    /// 命令行：继承系统用户权限，最高 `act`（信任变更走界面）。
    pub fn cli() -> Self {
        Self::unrestricted(Origin::Cli, Tier::Act, false)
    }

    /// 界面：`human_present` 为真时才可能进入 `trust`。
    pub fn ui(human_present: bool) -> Self {
        let max = if human_present {
            Tier::Trust
        } else {
            Tier::Act
        };
        Self::unrestricted(Origin::Ui, max, human_present)
    }

    /// 测试台：不限，但不假装人在场。
    pub fn test() -> Self {
        Self::unrestricted(Origin::Test, Tier::Trust, false)
    }

    pub fn allow_trust(mut self) -> Self {
        self.max_tier = Tier::Trust;
        self
    }

    pub fn with_read_root(mut self, root: impl AsRef<Path>) -> Self {
        if let Scope::Only(set) = &mut self.read_roots {
            set.insert(root.as_ref().to_path_buf());
        }
        self
    }

    pub fn with_allowed_target(mut self, target: impl Into<String>) -> Self {
        if let Scope::Only(set) = &mut self.allowed_targets {
            set.insert(target.into());
        }
        self
    }

    pub fn with_max_send_bytes(mut self, bytes: u64) -> Self {
        self.max_send_bytes = Some(bytes);
        self
    }
}
