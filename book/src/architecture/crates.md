# 分层与 crate

原则：**宁深勿多**。每个 crate 是一个深模块——小接口，背后大量实现（见 mattpocock/skills 的 codebase-design）。
第一代有 13 个 crate，其中不少是薄薄的转手层；第二代控制在 10 个左右。

## 分层

低层不许依赖高层，由 `cargo xtask layers` 在 CI 里检查。界面相关的库（egui/Slint、UniFFI、rmcp）只许出现在 L5。

| 层 | crate | 职责 | 可以做 I/O 吗 |
|---|---|---|---|
| L0 | `aa4c-types` | 共享数据类型：指纹、档位、错误码、命令描述 | 否 |
| L0 | `aa4c-wire` | 线路协议：帧、消息、版本协商（纯编解码，可 fuzz）| 否 |
| L1 | `aa4c-id` | 个人/设备密钥、设备证书、吊销、核对码、空间成员链的签名与校验 | 否（密钥存储经 L1 的 state 注入）|
| L1 | `aa4c-state` | redb 状态库 + `config.toml` + 安全存储适配（钥匙串等）| 本地磁盘 |
| L2 | `aa4c-net` | QUIC 端点、信标、连接阶梯、端口映射、打洞、中转站、远程访问的端口转发，以及本地 IP 链路的公共状态机（客户端与服务端）| 网络 |
| L3 | `aa4c-files` | 发送/接收/续传、共享文件夹同步、一次性发送链接、扫码收发页的 HTTP 服务 | 网络 + 磁盘 |
| L3 | `aa4c-plugins` | 独立的 Wasm 解析/执行沙箱、manifest 与意图校验；可单独 fuzz，M8 前不是必选依赖 | — |
| L4 | `aa4c-engine` | Session、命令注册表、`CallerContext`、事件流、能力检查；把插件意图接到命令策略 | — |
| L5 | `apps/aa4c` | 无界面版：常驻、CLI、本地 IPC、MCP、中转站 | — |
| L5 | `apps/aa4c-desktop` | 桌面外壳（只调命令）| — |
| L5 | `ffi/aa4c-mobile` | UniFFI 绑定，供 Kotlin/Swift 薄壳调用；**唯一允许 `unsafe` 的 crate**（UniFFI 生成代码需要）| — |
| — | `sdk/aa4c-plugin` | 插件作者用，编译到 `wasm32-unknown-unknown` | — |
| — | `xtask` | `layers` / `deps` / `ci` / `docs` / `release` 等开发工具 | — |
| — | `mobile/android`、`mobile/ios` | 手机薄壳工程（Gradle / Xcode）| — |

L0 与 `aa4c-id` 是纯计算，没有 I/O 和时钟；`aa4c-state` 只碰本地磁盘。它们是测试密度最高、最先写的部分。插件宿主单独成 crate，不让不可信 Wasm 解析器藏在引擎内部；
「约 10 个」是控制浅模块的方向，不是为了数字把不同风险边界硬塞在一起。

## 依赖预算

`apps/aa4c` 的运行时依赖（`cargo tree -p aa4c -e normal` 去重后）**≤ 130 个 crate**，`cargo xtask deps` 超出即失败。
第一代对照：`aa4c-core` 一项 188 个，Tauri 桌面壳 370 个。

新增依赖的 PR 必须写明：为什么需要、是否处理不可信输入、有没有 C/C++ 代码或 `unsafe`、维护状况、许可证。

「业务逻辑只用 Rust」不等于假装传递依赖里没有原生代码：加密 provider、系统 UI、钥匙串与平台 FFI 可能带 C/汇编或生成的绑定。
原则是**不引入外部业务进程和第二套业务语言**；任何原生传递依赖必须进入 `cargo xtask deps` 生成的清单并说明理由，
能用维护良好的纯 Rust 实现时优先纯 Rust。M0 实测后再为 rustls 选择 provider，不在设计阶段把 `ring` 写成无条件结论。

| 需求 | 选择 | 不选 |
|---|---|---|
| 传输 | quinn（QUIC）| TCP + tokio-rustls 平行路径 |
| TLS | rustls 原始公钥（crypto provider 由 M0 依赖/平台实验决定）| rcgen + x509-parser |
| 异步 | tokio（裁剪 features）| — |
| 签名 / 哈希 | ed25519-dalek、blake3 | — |
| 发现 | 自写组播信标（socket2；默认加密，见 ADR 0015）| mdns-sd |
| 状态 | redb（[ADR 0006](../../../docs/adr/0006-redb-not-sqlite.md)）| rusqlite（SQLite C 源码）、fjall（留给将来的 `folder_index`）|
| 授予的文件根 | cap-std + cap-fs-ext（[ADR 0020](../../../docs/adr/0020-cap-std-for-granted-roots.md)）| 自研 `openat` 沙箱、`canonicalize` 后普通打开 |
| 配置 | toml + serde | — |
| 线路消息 | serde_json | bincode 1 |
| 端口映射 | 自写最小 UPnP IGD 客户端 | 带完整 HTTP 栈的库 |
| 扫码收发 HTTP | 自写极小 HTTP/1.1（只服务一个页面）或 `tiny_http` | hyper/axum 全家桶 |
| CLI | clap + clap_complete + clap_mangen | — |
| MCP | rmcp（只在 `apps/aa4c`）| — |
| 插件 | wasmi | wasmtime、mlua |
| 手机桥接 | UniFFI | 手写 JNI / C ABI |
| 近场平台能力 | 系统 API + 薄平台适配器；建链后复用现有 QUIC / HTTP | 为每种无线方式引入一套跨平台业务框架 |
| 文件监听 | v1 不用，定时扫描 + 手动刷新 | notify（平台 C 绑定）|
