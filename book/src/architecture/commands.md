# 命令与事件

## 一切皆命令

每个用户可见的行为都是引擎命令注册表里的一个**命令**。桌面版、手机版、`aa4c` 命令行、MCP、插件都只调命令，
不各自实现业务逻辑。测试也打在这里——命令注册表是唯一的接缝，也是测试面。

```text
调用方（任意外壳）
   │  命令 id + JSON 参数 + 入口生成的 CallerContext
   ▼
命令注册表 ── 参数校验 · 能力/文件根/人类步骤检查 ──▶ 命令实现
                                         │
                                         ▼
                              Session（身份、状态、网络、插件）
                                         │
                                 结果 JSON / 错误 · 事件
```

## 命令描述

| 字段 | 说明 |
|---|---|
| `id` | 点分的稳定标识，如 `send.files`、`contacts.invite.create`、`spaces.member.add` |
| `summary` | 一句话说明（同时用于 CLI 帮助与 MCP 工具描述）|
| `params` | 参数 JSON Schema |
| `result` | 结果 JSON Schema |
| `tier` | 能力级别：`read` / `act` / `trust`（[ADR 0011](../../../docs/adr/0011-trust-changes-are-human-only.md)）|
| `effects` | 可机器读取的副作用：`none` / `local_write` / `network` / `trust`，可多选 |
| `path_access` | `none` / `read` / `write` / `read_write`；入口据此应用授予的根目录 |
| `long_running` | 是否立即返回一个可订阅进度的 id（发送类即 Transfer id）|
| `human_step` | 是否包含必须由人完成的步骤（如比对核对码）|
| `idempotent` | 同一幂等键重试是否安全；长任务命令必须说明 |

`aa4c commands --json` 输出全部命令描述，是 agent 了解 AA4C 能做什么的唯一入口。文档可以解释命令，但不能另列一套会漂移的参数表。

## CallerContext

`CallerContext` 至少包含：`origin`（UI / CLI / MCP / plugin / test）、允许的最高能力级别、只读根目录、可写根目录、
允许的接收目标、分享与端口转发能力、单次/会话字节预算、`human_present`、请求 id 与截止时间。它由可信入口创建；
JSON 参数里出现同名字段一律忽略或拒绝。

- UI 的人在场操作可以进入 `trust`，但仍要完成核对码、系统验证等人类步骤。
- MCP 默认最高到 `act`，但发送还必须命中启动时授予的接收目标与字节预算；创建无指定接收人的一次性链接另需显式 `allow_share`，
  打开端口转发另需显式 `allow_tunnel`。
- 插件只能继承安装时批准的 `read`/`act` 子集，永远不能进入 `trust`。
- 普通 CLI 继承启动它的系统用户权限；给 agent 使用时优先 MCP。CLI 的 `--json` 只是输出格式，不是安全沙箱。

能力级别必须是**静态的**，不能让同一个命令「联系人时是 act、陌生人时变成 trust」，否则 MCP 的工具清单会说谎。
需要不同权限的行为拆成不同命令。

## 命令一览（v1 草案）

| 命名空间 | `read` | `act` | `trust` 👤 |
|---|---|---|---|
| `me` | `me.show` | `me.rename` | `me.recovery.export` |
| `devices` | `devices.list` | — | `devices.join.start`、`devices.join.accept`、`devices.revoke` |
| `nearby` | `nearby.list` | — | `nearby.receive_mode.set` |
| `proximity` | `proximity.routes` | `proximity.send`、`proximity.handoff` | — |
| `contacts` | `contacts.list` | `contacts.rename` | `contacts.add.qr`、`contacts.invite.create`、`contacts.invite.accept`、`contacts.invite.approve`、`contacts.remove`、`contacts.settings.set` |
| `send` | — | `send.files`、`send.text`、`send.cancel`、`send.retry` | — |
| `inbox` | `inbox.list` | `inbox.accept`（已有关系）、`inbox.reject`、`inbox.open` | `inbox.accept_stranger` |
| `transfers` | `transfers.list`、`transfers.show` | — | — |
| `share` | `share.list` | `share.create`、`share.revoke` | `share.accept` |
| `webdrop` | — | `webdrop.start`、`webdrop.stop` | — |
| `spaces` | `spaces.list` | `spaces.create` | `spaces.invite.accept`、`spaces.member.add`、`spaces.member.remove`、`spaces.member.role`、`spaces.leave` |
| `folders` | `folders.list`、`folders.browse` | `folders.create`、`folders.fetch`、`folders.camera_upload.set` | — |
| `relay` | `relay.status` | `relay.enable`、`relay.disable` | `relay.social.set` |
| `services` | `services.list` | — | `services.publish`、`services.unpublish` |
| `tunnel` | `tunnel.list` | `tunnel.open`、`tunnel.close` | — |
| `status` | `status.show`（本机 + 每台设备的可达性与档位）| — | — |
| `config` | `config.get` | `config.set` | — |
| `plugin` | `plugin.list` | — | `plugin.install`、`plugin.remove` |

👤 = 信任级，MCP 默认不暴露。插件声明的命令 `plugin.<id>.<name>` 只能是 `read` 或 `act`。

- `contacts.invite.accept` 是 B 打开邀请后的确认，`contacts.invite.approve` 是 A 看到对方名字与核对码后的确认；
  `spaces.invite.accept` 是受邀者接受空间邀请。
- `contacts.settings.set` 管免确认与能否经我的中转站中转，所以是 `trust`；只改备注名用 `contacts.rename`。
- `share.accept` 收的是没有关系的人发来的内容，与 `inbox.accept_stranger` 同级。
- `services.publish` 把本机服务开放给我的其他设备，所以是 `trust`；`tunnel.open` 只能连我的设备上已开放的服务
  （[远程访问](../network/remote-access.md)）。
- `config.set` 的 `path_access = write`：路径类配置键（如 `receive.save_to`）按调用者的可写根目录检查。
  接收模式等信任设置不在配置里，`config.set` 改不到。
- `proximity.send` 只走 AA4C 直连或扫码收发，必须给出明确的路径 id，不接受「自动」；平台直传是单独的 `proximity.handoff`，
  描述里静态标记 `human_step = true`，设备选择、配对和系统确认不能由 MCP 代替。路径在执行前消失时返回 `unsupported`，
  不得静默换到安全语义更弱的路径。

## 错误

错误是结构化的：`{"code": "unreachable", "message": "…", "hint": "…", "retryable": true, "details": {…}}`。
`code` 是稳定枚举（`invalid_params`、`not_found`、`unreachable`、`rejected`、`needs_human`、`permission_denied`、
`busy`、`unsupported`、`io`、`internal`），各外壳据此给出人话提示，CLI 据此决定退出码（[无界面版 aa4c](../surfaces/cli.md)）。

## 事件

事件是引擎推送的 JSON 对象，`{"type": "transfer.progress", "at": …, "data": {…}}`。主要类型：

`nearby.changed`、`proximity.routes_changed`、`device.reachability`（含档位与失败原因）、`inbox.request`、`transfer.progress`、`transfer.done`、
`transfer.failed`、`contact.request`（有人兑现我的邀请，等我确认）、`contact.added`、`space.invite`、`space.changed`、`folder.changed`、
`relay.status`、`tunnel.changed`、`plugin.notify`。

外壳订阅事件来刷新界面；CLI 用 `aa4c events` 输出为 NDJSON；MCP 以通知或资源的形式提供。

## 永不崩溃

命令参数和网络输入都是敌意输入。命令实现必须对**任何**参数和状态返回 `Err` 而不是 panic；
`panic_hunt` 测试用对抗性参数（缺字段、类型错、超范围、超长、畸形 Unicode）轰炸每一个命令。
外壳在命令分发处有最后一道兜底：捕获逃逸的 panic，报告为 `internal` 错误，引擎继续运行。兜底不能代替命令自己的错误处理；
可能以 abort 结束进程的第三方解析器还要靠版本固定、资源上限、回归样本和必要时的进程隔离防护。
