# 无界面版 aa4c（命令行）

一个程序 `aa4c`，四种用途：**常驻服务**（NAS、服务器上无界面运行）、**命令行客户端**、**中转站**、**MCP 服务**。
单个静态二进制，另有 Docker 镜像（`FROM scratch`）。

## 子命令

```text
aa4c up [--foreground]               启动常驻引擎（无界面）；down 停止
aa4c status [--json]                 本机状态 + 每台设备的可达性与档位 + 连不上的原因
aa4c me                              我的名字与指纹
aa4c nearby                          附近的人与设备
aa4c proximity [routes|send|handoff] 当面可用路径；send 走 AA4C 或扫码，handoff 交给系统分享
aa4c send <路径>... --to <人或设备> [--wait]
aa4c send --text "…" --to <人或设备>
aa4c inbox [list|accept|reject]
aa4c contacts [list|invite|remove]
aa4c devices [list|join|revoke]      join/revoke 需要人操作，见下
aa4c spaces [list|create|members]
aa4c folders [list|browse|fetch]
aa4c share [create|list|revoke|accept]
aa4c webdrop [start|stop]
aa4c relay [status|enable|disable]
aa4c services [list|publish|unpublish] 本机开放给我的其他设备的服务（远程访问）
aa4c tunnel <设备> <服务> [--local-port <端口>]   在 127.0.0.1 上开端口转发
aa4c connect <设备> <服务>            标准输入输出接到服务，给 ssh 的 ProxyCommand 用
aa4c ssh-config                      打印开放了 SSH 的我的设备的 ssh 配置段
aa4c config [get|set]
aa4c events                          NDJSON 事件流
aa4c commands [--json]               全部命令及参数/结果 Schema、能力级别、副作用、路径访问
aa4c run <命令 id> --params '{…}'    通用入口：任何命令（含插件命令）
aa4c mcp [--read-root <目录>] [--write-root <目录>] [--allow-to <人或设备>] [--max-send-bytes <数>] [--allow-share] [--allow-tunnel] [--allow-trust]
aa4c plugin [list|install|remove]
```

每个子命令都是一个命令 id 的薄映射；`aa4c run` 可以调用任何命令。`connect` 是 `tunnel.open` 接到标准输入输出的形式，
`ssh-config` 是 `services.list` 的另一种输出格式。`--help` 是可执行的真相来源，man page 与 shell 补全由代码生成。

`proximity send` 复用普通发送或扫码收发；`proximity handoff` 会打开系统分享面板，需要人在场，无界面设备结构化返回
`unsupported`，不会猜一个无线方式或静默降级。

给 agent 使用时以 `aa4c commands --json` 为运行时真相，不要让 skill 或文档复制完整参数表。需要限制文件范围或处理不可信内容时使用
`aa4c mcp --read-root … --write-root …`；直接 CLI 与当前系统用户权限相同，不提供额外沙箱。

## 输出与退出码约定

- 默认输出给人看的文本；`--json` 输出单个 JSON 对象。长任务加 `--wait` 时逐行输出进度事件（NDJSON），最后一行是结果。
- 错误同样结构化：`{"error":{"code":"unreachable","message":"…","hint":"…"}}`。
- 退出码固定：

| 码 | 含义 |
|---|---|
| 0 | 成功 |
| 1 | 运行错误 |
| 2 | 用法错误（参数错、缺参数、互斥参数同时出现）|
| 3 | 找不到（人、设备、文件、命令）|
| 4 | 连不上 |
| 5 | 需要人确认，或权限不足 |

- 标准输出不是终端时**从不交互提问**；破坏性操作支持 `--dry-run`。
- 环境变量：`AA4C_HOME`（数据目录）、`AA4C_DEVICE_NAME`（首次启动时的设备名）。

## 需要人的步骤

加入设备、当面加联系人要比对核对码。命令行里的做法：`aa4c devices join` 打印加入码（文本形式的二维码内容 + 6 位数字）与核对码，
等待用户在另一台设备上确认；命令本身不能替人确认。

## 无界面设备的首次设置

`aa4c up` 第一次运行时生成身份，并在标准输出打印加入码（不写入 `logs/`，5 分钟过期，过期后用 `aa4c devices join` 重新生成）。在手机或电脑上「加入我的设备」输入它，之后这台机器就是「我的设备」，
后续管理都在手机或电脑上完成（[关键流程](../identity/flows.md)第 8 条）。
