# 架构决定记录（ADR）

只为同时满足三个条件的决定写 ADR：**难回头**、**不看上下文会觉得奇怪**、**有真实的取舍**。
容易撤销的、显而易见的、没有替代方案的，不写。

## 格式

文件名 `NNNN-slug.md`，编号递增。正文通常一到三段：背景、决定、理由。只在真有价值时加：

- `状态`：提议中 / 已接受 / 已废弃 / 被 ADR-NNNN 取代
- `考虑过的方案`：只在被否决的方案值得记住时
- `后果`：只在下游影响不明显时

推翻一条 ADR：写一条新的，旧的状态改为「被 ADR-NNNN 取代」，不要直接改旧的结论。
状态为「提议中」的 ADR 在实验或评审得出结论前可以直接修订；「已接受」之后按上面的规则。

## 索引

| 编号 | 决定 | 状态 |
|---|---|---|
| [0001](0001-rust-engine-native-mobile-shells.md) | Rust 引擎 + 原生手机薄壳，不用 webview | 已接受 |
| [0002](0002-person-device-contact-space.md) | 人 · 设备 · 联系人 · 空间，不要账号 | 已接受 |
| [0003](0003-stranger-send-and-receive-mode.md) | 附近的人一次性发送与接收模式 | 已接受 |
| [0004](0004-quic-only.md) | 只用 QUIC | 已接受 |
| [0005](0005-raw-public-keys.md) | 原始公钥代替 X.509 | 已接受（macOS 实测，三平台待 CI）|
| [0006](0006-redb-not-sqlite.md) | redb + config.toml 代替 SQLite | 已接受（附 M0 验证条件）|
| [0007](0007-json-control-messages.md) | 线路控制消息用 JSON | 已接受 |
| [0008](0008-wasm-plugins-not-lua.md) | 插件用 wasmi，不用 Lua | 已接受 |
| [0009](0009-no-downloader-no-ai.md) | 下载器与 AI 不在范围内 | 已接受 |
| [0010](0010-self-hosted-relays-only.md) | 只用自建中转站，含社交中转 | 已接受 |
| [0011](0011-trust-changes-are-human-only.md) | 信任变更只能由人完成 | 已接受 |
| [0012](0012-aa4c-com-static-only.md) | aa4c.com 只托管静态页面 | 已接受 |
| [0013](0013-not-iroh.md) | 不用 iroh | 已接受 |
| [0014](0014-clean-break-from-v1.md) | 与第一代断代重建 | 已接受 |
| [0015](0015-private-nearby-discovery.md) | 默认信标不暴露稳定身份 | 已接受（帧格式见 ADR 内的实验结果）|
| [0016](0016-no-remote-browser-share-in-v1.md) | 远程接收必须安装；近场可免安装直传 | 已接受 |
| [0017](0017-remote-access-by-service-forwarding.md) | 远程访问用服务转发，不做插件、不做虚拟网卡 | 已接受 |
| [0018](0018-agents-act-directly-across-my-devices.md) | Agent 在我的设备之间直接动手 | 已接受 |
| [0019](0019-apache-2-license.md) | 许可证用 Apache-2.0 | 已接受 |
| [0020](0020-cap-std-for-granted-roots.md) | 授予的文件根用 cap-std，不自研 openat 沙箱 | 已接受（附 M0 验证条件）|
