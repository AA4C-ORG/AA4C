# 存储

> 决定见 [ADR 0006](../../../docs/adr/0006-redb-not-sqlite.md)（提议中）。

## 数据目录

用 `dirs` 取平台目录，可由环境变量 `AA4C_HOME` 覆盖（测试与同机多实例用）：

| 平台 | 位置 |
|---|---|
| macOS | `~/Library/Application Support/com.aa4c.AA4C/` |
| Windows | `%APPDATA%\AA4C\` |
| Linux | `$XDG_DATA_HOME/aa4c/`（默认 `~/.local/share/aa4c/`）|
| Android / iOS | App 私有目录 |
| Docker | `/data`（挂载卷）|

内容：

```text
config.toml        人类可读的配置（引擎启动时校验；运行时通过 config 命令修改）
state.redb         状态库
engine.lock        单实例锁
engine.sock        本地 IPC（Unix）
plugins/           已安装的插件（.wasm + manifest）
logs/              日志（不含私钥、文件内容、完整公钥）
```

收到的文件默认落在系统「下载」目录下的 `AA4C/`，可在配置里改。

## 密钥

个人私钥与设备私钥**不进 redb**，放系统安全存储：macOS / iOS 钥匙串、Android Keystore（包装密钥）、Windows DPAPI、
Linux Secret Service；不可用时（如无桌面环境的 Linux / Docker）退回数据目录下的文件，权限 0600，并在 `aa4c status` 里提示。

## 状态库（redb）表

| 表 | 内容 |
|---|---|
| `meta` | 数据格式版本、引擎版本 |
| `me` | 我的个人公钥、本机设备证书、我的设备证书列表、我签发的吊销 |
| `contacts` | 联系人：个人公钥、备注名、设置、已知设备证书、最近地址、可用中转站提示 |
| `revocations` | 收到的吊销（按个人公钥分组）|
| `spaces` | 空间：成员链（全部版本）、共享文件夹定义 |
| `folder_index` | 共享文件夹的文件元数据索引（路径、大小、修改时间、BLAKE3、所在设备）|
| `transfers` | 传输记录 |
| `inbox` | 待确认与已收到的条目 |
| `shares` | 分享链接（口令摘要、目标、有效期、访问记录）|
| `addresses` | 设备地址缓存（上次已知地址、来源、时间）|

数据格式升级：引擎启动时读 `meta` 里的版本，按顺序执行 Rust 写的升级步骤；每个升级步骤都有「旧数据 → 新数据」的测试。
升级前自动备份 `state.redb`。

## 配置（config.toml）示例

```toml
[me]
device_name = "霍涛的 MacBook"

[receive]
mode = "contacts_only"        # contacts_only | everyone | off；everyone 由引擎自动在 10 分钟后改回
save_to = "~/Downloads/AA4C"

[network]
remote = true                 # 允许跨网连接
port_mapping = true           # 在 remote = true 时自动在路由器上开端口
port = 0                      # 0 = 自动选择

[relay]
enabled = false
serve = "my_devices"          # my_devices | 另见社交中转设置（存在状态库，属信任级）
max_bandwidth_mbps = 20
```

信任相关的设置（社交中转为谁开放、联系人免确认）不放 `config.toml`，只能经 `trust` 级命令修改，防止被脚本或 agent 直接改文件绕过。

`config.toml` 的可读性用于排查、备份与引擎停机时的恢复，不代表推荐绕过命令直接改。引擎运行时只用 `config.get` / `config.set`；
写入采用临时文件 + fsync + 原子替换。直接编辑若与引擎并发，可能被拒绝或在下次保存时覆盖。
