# 0020 授予的文件根用 cap-std，不自研 openat 沙箱

状态：已接受（2026-10-30）

MCP 会话只能读 `--read-root`、写 `--write-root`（[MCP 与 AI agent](../../book/src/surfaces/agents.md)）。
「先 `canonicalize` 检查、再用普通路径打开」有检查与使用之间的竞态：检查之后目录被换成指向根外的符号链接，打开的就是根外的文件。
正确做法是启动时把根目录打开成目录句柄，此后一切访问都相对这个句柄。

决定：**用 cap-std 4.x 的 `Dir` 作为授予的根**，配合同一项目的 `cap-fs-ext` 逐组件以不跟随符号链接的方式打开，
保持「拒绝符号链接组件」的严格策略（cap-std 默认允许留在根内的链接）。

- `ambient_authority()` 只许出现在一处：MCP 启动时接纳用户授予根目录的入口。引擎与命令层只拿到封装后的 `GrantedRoot`
  （`open_read`、`create_new`、`create_dir`、`atomic_write`、`read_dir`），拿不到原始路径；命令参数只传根 id 与相对路径。
- MCP 授权路径上禁用 `std::fs` 的按路径访问；先做词法拒绝（绝对路径、`..`、盘符、UNC），再由句柄级约束兜底。
- 写入用同目录临时文件加原子重命名。
- 只在某平台缺少必要操作时，才对那一项操作用 `rustix` 等平台 API 补充，不整体改成自研沙箱。
- 必须固定在已修复 Windows 特殊设备名绕过的版本（3.4.1 及以后）。

**为什么不是 `openat`**：普通 `openat` 只约束最后一个组件，中间的符号链接仍会被跟随；`openat` crate 只有 Unix 接口、仍标 Beta、
2021 年起不再更新。要安全就得自己写 Linux 的 `openat2`、macOS/iOS 的逐组件 `openat`、Windows 的目录句柄与重解析点处理，
再配每个平台的竞态测试。这是一个敏感的自研安全模块，不值得为省几个 crate 去承担。cap-std 多出的依赖（`cap-primitives`、`rustix` 等）
是值得花的依赖预算，进入 `cargo xtask deps` 清单并逐项说明。

**M0 验证条件**：同一套越权测试在 Linux、macOS、Windows 上都过——`../`、绝对路径与盘符/UNC/`\\?\`、Windows 设备名
（`CON`、`NUL`、`COM1`）、末尾与中间的符号链接、指向根内与根外的链接、检查过程中把目录反复换成链接、junction 等重解析点、
目标已存在且是链接、写入中途目标被替换、授权目录启动后被改名。

**残余边界**：路径沙箱认不出硬链接。授权目录内若有指向其他 inode 的硬链接，它在文件系统意义上就是授权目录里的文件；写进[安全与威胁模型](../../book/src/security.md)。
