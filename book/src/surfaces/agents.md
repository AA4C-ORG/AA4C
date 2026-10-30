# MCP 与 AI agent

AA4C 要让 AI agent 能直接使用：「把这份报告发到我手机上」「NAS 上的家庭相册还有多少空间」「这台电脑连得上家里的 NAS 吗」。
同时不能让 agent 成为攻击者绕过「信任由人建立」的通道。

## 怎么接入

- **命令行**：agent 可以在终端里调 `aa4c`，用 `--json` 读结果、看退出码（[无界面版 aa4c](cli.md)）。CLI 继承系统用户权限，
  不是 agent 沙箱；自动化优先使用带根目录授权的 MCP。`aa4c commands --json` 列出全部能力。
- **MCP**：`aa4c mcp` 在标准输入输出上提供 MCP 服务，工具由命令注册表自动生成（工具名 = 命令 id，描述 = summary，参数 = JSON Schema）；
  另提供资源 `aa4c://status`、`aa4c://devices`、`aa4c://contacts`。它连到正在运行的引擎（桌面版或 `aa4c up`），没有则提示先启动。
- **Skill**：仓库提供符合 Agent Skills 目录约定的 `skills/aa4c/SKILL.md` 和 `agents/openai.yaml`。Skill 只写选择与安全流程，
  不复制命令参数；运行时以命令注册表为准。各宿主的插件 manifest 只在真实安装链路通过测试后再加，避免先承诺一个未经验证的「一键安装」。

注册示例：

```bash
claude mcp add aa4c -- aa4c mcp \
  --read-root ~/Documents \
  --write-root ~/Downloads/AA4C \
  --allow-to "我的手机" \
  --max-send-bytes 1073741824
```

## 安全边界

[ADR 0011](../../../docs/adr/0011-trust-changes-are-human-only.md)：

| 级别 | 例子 | MCP 默认 |
|---|---|---|
| `read` | 看状态、设备、联系人、收到的、空间内容 | 暴露 |
| `act` | 发送、取回、生成一次性发送链接（另需 `--allow-share`）、打开端口转发（另需 `--allow-tunnel`）、扫码收发、读取近场路径、接受**已有关系**的人发来的文件 | 暴露 |
| `trust` | 加入设备、添加/删除联系人、生成邀请链接、改空间成员与角色、吊销设备、开放远程访问的服务、打开「所有人」接收、开关社交中转、安装插件、接受陌生人或一次性发送链接发来的文件 | **不暴露**，需 `--allow-trust` |

- 即使打开 `--allow-trust`，需要比对核对码的步骤也只能由人完成。
- `proximity.handoff`（系统直传）虽是 `act`，但描述里静态标记 `human_step = true`：agent 可以读取 `proximity.routes` 并准备文件，
  不能替人在系统分享面板里选择设备、完成系统配对，或把系统的「已提交」解释成「对方已收到」。
- `inbox.accept` 只接受已有关系的请求；陌生人的请求走单独的 `inbox.accept_stranger`（trust），一次性发送链接的内容走
  `share.accept`（trust），避免同一工具运行时改变级别。
- **文件根目录**：MCP 会话只能发送 `--read-root` 下的文件、只能把收到的东西放进 `--write-root`；未授予的根目录视为无权限，
  路径一律相对预先打开的根目录句柄解析，拒绝 `..`、绝对路径和符号链接组件。不能用「先 canonicalize、再普通 open」的竞态实现授权；
  M0 需比较 `cap-std` 与各平台 `openat`/等价安全打开方式的依赖和正确性。
- **目标与限额**：发送只能去 `--allow-to` 明确授予的人或设备，并受单次与会话字节预算约束；没有 `--allow-share` 时不暴露一次性链接创建。
  名字在 MCP 启动时必须唯一解析并固定到指纹，重名就拒绝启动并要求用指纹。单次请求大小、批量步数、回复大小也都有上限。
  这样文件里的提示注入不能把「发到我手机」改成「发给另一个联系人」。
- 工具描述与文件内容都是不可信数据：MCP 工具的 Schema 帮助正确调用，但**不是**授权边界；授权只由能力级别与根目录决定。
- 每次调用的 `CallerContext` 由 MCP 服务端生成。客户端参数不能声明自己是 UI、有人在场或拥有更高能力。

## 给 agent 的输出

- 错误带 `hint` 字段，用一句话说明下一步可以做什么（「对方不在线；可等待上线后自动重试」）。提示不能推荐一个使用同样不可达链路的假替代方案。
- 长时间运行的发送返回 Transfer id，用 `transfers.show` 查询或订阅 `transfer.progress` 事件。
- 不在输出里放私钥、完整公钥、文件内容；指纹只给缩写形式。
