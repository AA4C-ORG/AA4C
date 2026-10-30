# 手机版

Android（先）与 iOS。照片在手机上，手机是核心场景的一端。

## 技术

- **原生薄壳 + Rust 引擎**（[ADR 0001](../../../docs/adr/0001-rust-engine-native-mobile-shells.md)）：
  Android 用 Kotlin + Jetpack Compose，iOS 用 Swift + SwiftUI；引擎经 UniFFI 生成的绑定调用。
- 薄壳只做两件事：**画界面**、**接系统能力**（分享菜单、相册、通知、后台任务、权限、扫码相机、点对点网络与系统交付）。
  它只报告能力并执行 Rust 给出的计划，业务选择策略一行都不写。
- 薄壳调用的命令与桌面版完全相同；薄壳的界面状态尽量也由引擎事件驱动。

## 系统集成（决定用户体验的部分）

| 能力 | Android | iOS |
|---|---|---|
| 从别的 App 分享到 AA4C | 分享 Intent 目标 | 共享扩展（Share Extension）|
| 选照片 | Photo Picker | PHPicker |
| 相册自动上传到共享文件夹 | 前台服务 + WorkManager | 后台任务（BGProcessingTask）+ 「打开 App 时补传」|
| 局域网发现 | 需持有 MulticastLock | 需「本地网络」权限说明 |
| 无共同路由器时建立本地 IP | Wi-Fi Direct / 临时热点（运行时探测）| Network framework 点对点 Wi-Fi / Wi-Fi Aware（按系统与硬件能力）|
| 系统直传 | 系统分享目标、经典蓝牙等公开能力；不可用就隐藏 | 系统分享与点对点能力；不可用就隐藏 |
| 收到请求 | 通知（带接收/拒绝按钮）| 通知；App 在后台时接收受系统限制 ⚠ |
| 扫码 | 相机扫码（加入设备、加联系人、扫码收发）| 同 |
| 打开邀请/分享链接 | App Links（`https://aa4c.com/i/`、`/s/`）| Universal Links |

这些能力由 Kotlin / Swift 薄壳接系统 API，但路线选择、文件授权、大小预算和状态机仍在 Rust。蓝牙、红外、Wi-Fi Direct 等
不能按平台名称硬编码为“必定可用”，必须向引擎报告运行时能力与不支持原因。

⚠ **iOS 后台限制**：App 不在前台时，iOS 不允许长期保持监听，「手机在口袋里也能收到电脑发来的文件」在 iOS 上做不到完全可靠。
方案与取舍列在[待细化的问题](../open-questions.md)。

## 第一代可以复用的经验

第一代 Android 已踩过：`MulticastLock` 获取与释放、`compileSdk` 与 build-tools 版本、代理环境下 SDK 下载截断等，
见[从第一代学到的](../lessons.md)。

## 交付顺序

Android 在 M2（紧随桌面版附近互传之后），iOS 在 M6。M0 先做一个实验：Rust 引擎经 UniFFI 在 Android 模拟器上跑起来，
Kotlin 薄壳调用一个命令并收到事件。
