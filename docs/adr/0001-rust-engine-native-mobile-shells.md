# 0001 Rust 引擎 + 原生手机薄壳，不用 webview

状态：已接受（2026-10-29）

第一代用 Tauri + Vue + TypeScript + Kotlin 插件，外加 aria2、Transmission、llama-server 三个 C/C++ 外部程序：
`Cargo.lock` 648 个包，桌面壳一项 370 个依赖，`node_modules` 185 MB，四种语言。依赖与工具链的重量拖慢了每一次修改，
也让 AI agent 很难完整理解一个改动的影响面。

决定：**全部业务逻辑在一个 Rust 引擎里**。这约束我们写的应用代码与外部业务进程，不声称所有系统/加密传递依赖都没有 C、汇编或 FFI；
这些例外必须被依赖清单看见并逐项说明。桌面界面也用 Rust（egui 或 Slint，由 M0 原型决定）；不用 Tauri、Electron
或任何 webview，不写 JavaScript/TypeScript。**例外只有两处**：

1. **手机界面用 Kotlin / Swift 写薄壳**，经 UniFFI 调用 Rust 引擎。手机体验长在系统 API 上（分享菜单、相册、后台任务、
   通知、权限、点对点网络与系统文件交付），纯 Rust 的手机界面照样要原生胶水，iOS 尤甚；而手机是核心场景的一端，不能将就。
   薄壳只画界面、转调命令，并通过窄适配器报告和执行系统能力；选择策略、授权与状态机仍在 Rust，不在薄壳写业务逻辑。
2. **扫码收发页是纯 HTML**，由引擎临时提供，不用 JavaScript（下载是链接，上传是表单）。

考虑过的方案：纯 Rust 手机界面（egui/Slint on Android）——能跑，但系统集成与观感都差一截；先只做桌面——照片在手机上，
核心场景会缺一端。
