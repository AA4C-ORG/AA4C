# 插件

> 决定见 [ADR 0008](../../docs/adr/0008-wasm-plugins-not-lua.md)：WebAssembly + wasmi，不用 Lua。

## 插件能做什么

v1 刻意收窄：插件处理「某件事发生之后，接下来怎么办」。

- **钩子**：`on_receive`（默认只有元数据）、`on_folder_change`（共享文件夹有变化）。读取文件开头不是默认能力；插件必须申请
  `inspect_received_prefix`，人确认后宿主才会提供本次收到文件的前 64 KiB。
- **命令**：插件可以声明自己的命令，以 `plugin.<插件id>.<名字>` 注册进同一个命令注册表——于是自动出现在命令行、MCP、
  桌面版与手机版的「更多」里，agent 不需要为插件学任何新东西。

第一个官方示例插件：**归档规则**——按类型、扩展名、来源把收到的文件移到指定目录并打标签。

## 插件不能做什么

- 读写文件系统、联网、读时钟与随机数：**宿主不提供任何导入函数**，模块只要导入任何东西就拒绝加载。
- 所以下载器、需要自己联网的东西做不成插件——这与「AA4C 不是下载器」一致。
- v1 不允许插件用一个通用意图调用任意引擎命令。那会把「不能联网/不能读文件」变成可以间接绕过的文字游戏。

## 插件只返回意图

插件是一个纯函数：输入事件 JSON，输出**意图列表**，由引擎做能力检查后执行。

```json
[
  {"intent": "move_to", "file": "…", "dir": "照片/2026/10"},
  {"intent": "tag", "file": "…", "tags": ["旅行"]},
  {"intent": "notify", "text": "已归档 12 张照片"}
]
```

- `move_to` 只能处理钩子交给它的文件，只能移到安装时授予的目录内；插件不能凭路径读取别的文件。
- `notify` 有长度与频率限制；所有意图均在 `CallerContext(origin=plugin)` 下重新做能力与路径检查。
- 插件不能直接拿到结果再接着做（v1 不做多轮交互）。未来若增加命令调用，要为目标、参数与数据流设计可审计的能力，而不是只给命令 id 白名单。

## 安装与能力

`aa4c-plugin.toml`：

```toml
id = "archive-rules"
name = "归档规则"
version = "1.0.0"
abi = 1
hooks = ["on_receive"]

[[commands]]
name = "apply"
summary = "按规则整理「收到的」里现有的文件"
params = { type = "object", properties = { dry_run = { type = "boolean" } } }

[capabilities]
move_within = ["inbox", "pictures"]
inspect_received_prefix = false
```

`aa4c-plugin.toml` 是插件源码的一部分；SDK 构建时把它转成 manifest JSON 嵌进 `.wasm`。安装包只有一个 `.wasm` 时，
宿主只信模块导出的 manifest，不再同时读取一份可能漂移的 sidecar。
manifest 只申请 `inbox`、`pictures` 这类语义位置；安装界面把它们解析为本机目录并让人确认，不把开发者机器的绝对路径写进插件。

👤 安装是 `trust` 级操作：界面列出插件申请的全部能力，人确认后才装。升级时能力有变化要重新确认。

## 沙箱

- **wasmi**（纯 Rust 解释器）：没有 JIT，所有平台（含 iOS）可用。
- 每次调用一个新实例，调用之间不保留状态（需要状态时由插件在意图里请求引擎代为保存，v1 不做）。
- **fuel 指令预算** + 墙钟期限：死循环会被终止并报错。
- **内存上限**、递归深度上限、模块大小上限。
- 任何异常（畸形模块、trap、超预算、输出不是合法 JSON、意图越权）都只产生一个错误，绝不让引擎崩溃或卡住。
- 测试里有一组「恶意插件」：死循环、吃内存、递归爆栈、输出乱码、请求越权意图，全部必须被安全拒绝。
- 固定并审计 wasmi 版本，使用不会把 guest 深调用映射为宿主递归的 portable/loop 执行方式；模块解析和调用边界再加 panic 兜底。
  PhotoCraft 曾遇到[畸形模块触发 wasmi 除零 panic](https://github.com/storytold/photocraft/issues/9679)的真实案例，
  对应样本必须进入 AA4C 回归测试。若依赖仍可能 abort 而无法被兜底，插件宿主改为隔离进程，不拿「纯 Rust」交换主进程可靠性。

## ABI v1

核心 WebAssembly 模块（不用 component model，工具链太重），导出：

| 导出 | 作用 |
|---|---|
| `memory` | 唯一线性内存；宿主只在边界检查后读写 |
| `aa4c_abi_version() -> i32` | 返回 1 |
| `aa4c_alloc(len: i32) -> i32` | 宿主写入参数用；每次调用是新实例，不要求释放 |
| `aa4c_manifest() -> i64` | `(len << 32) | ptr`，返回嵌入的 manifest JSON |
| `aa4c_handle(ptr: i32, len: i32) -> i64` | `(len << 32) | ptr`，输入事件 JSON，输出意图列表 JSON |

指针和长度都先做边界与溢出检查；manifest 和输出大小有独立上限。`sdk/aa4c-plugin` 提供一个宏把这些样板藏起来，
插件作者只写 `fn on_receive(event) -> Vec<Intent>`。
任何能编译到 `wasm32-unknown-unknown` 的语言都能写插件。
