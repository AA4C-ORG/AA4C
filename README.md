# AA4C

**把文件 AA 给身边和远方的人，也让你自己的设备彼此连得上。**

AA4C（AA连接，AA for Connection）是一个开源的点对点文件互传与设备互联工具：不需要账号，没有官方服务器。
设备之间优先直连；直连不通时，只经你自己选择的中转站转发端到端加密的数据。

[![Status: design](https://img.shields.io/badge/status-design-orange)](book/src/roadmap.md)
[![Engine: Rust](https://img.shields.io/badge/engine-Rust-b7410e)](book/src/architecture/overview.md)
[![Docs: design book](https://img.shields.io/badge/docs-design%20book-blue)](book/src/SUMMARY.md)

> [!NOTE]
> 项目处于**设计阶段**：仓库目前只有设计文档，还没有可运行的代码和安装包。实现按[路线图](book/src/roadmap.md)推进，欢迎参与设计讨论。

## 目录

- [适用场景](#适用场景)
- [功能](#功能)
- [设计原则](#设计原则)
- [工作原理](#工作原理)
- [文档](#文档)
- [路线图](#路线图)
- [参与贡献](#参与贡献)
- [安全](#安全)
- [许可证](#许可证)
- [联系](#联系)

## 适用场景

- **身边的人**：聚会、办公室里把照片和文件发给旁边的人。对方装了 AA4C 就加密直连；没装也能扫码在浏览器里接收。
- **自己的手机和电脑**：照片、文件、一段文字或一个链接，随手互传，不用数据线，也不绕道聊天软件。
- **远方的家人朋友**：加一次联系人，以后不在同一个网络也能直接发送大文件。
- **一家人、一个小团队**：共享文件夹，全家手机相册自动汇到家里的电脑或 NAS。
- **自己的几台电脑和 NAS**：在单位或外面 SSH、远程桌面到家里的设备。

## 功能

v1.0 计划提供以下功能，范围以[核心功能与范围](book/src/product/features.md)为准：

| 功能 | 说明 |
|---|---|
| 附近互传 | 自动发现同一网络里的人和设备，发送文件、文件夹、文字与链接；陌生人发来的内容每次都要确认 |
| 免安装接收 | 对方没装 AA4C 时扫码在浏览器里收发；平台支持时也可交给系统分享（AirDrop、Quick Share 等）|
| 我的设备 | 手机、电脑、NAS 加入同一个「我」，彼此互传无需确认 |
| 联系人与跨网发送 | 加一次联系人即认得对方的所有设备；跨网络时优先直连，必要时经自建中转站 |
| 一次性发送链接 | 不加联系人也能跨网络发送一次；接收方需安装 AA4C，网站不保存任何文件 |
| 共享空间 | 家庭或小团队的共享文件夹，按角色（所有者、可编辑、只读）在成员设备之间同步 |
| 常开设备与中转站 | NAS、小主机无界面运行，一键成为中转站，可在手机或电脑上远程管理 |
| 远程访问 | 把自己设备上的 SSH、远程桌面等服务开放给自己的其他设备 |
| 命令行与 AI agent | `aa4c` 命令行与 MCP 覆盖全部功能，输出结构化 JSON；改变信任关系的操作只能由人完成 |
| 插件 | WebAssembly 沙箱插件，处理「收到文件之后怎么办」这类自动化 |

明确不做：下载器、AI 与知识库、聊天、云盘、账号体系、社区（[ADR 0009](docs/adr/0009-no-downloader-no-ai.md)）。

## 设计原则

- **数据属于用户**：没有中心存储；中转站只转发端到端密文，不保存文件内容。
- **不依赖服务商**：没有账号服务器，没有官方中转站；aa4c.com 只托管静态页面。
- **默认安全**：AA4C 设备之间的传输全程端到端加密；信任关系必须由人亲手建立；默认只接收联系人发来的东西。
- **轻量**：业务逻辑集中在一个 Rust 引擎里，依赖数量有预算，并在 CI 中检查。
- **如实告知**：连不上时说清卡在哪一步、为什么、可以怎么办，不做超出能力的承诺。

## 工作原理

```text
桌面版 · 手机版 · aa4c（命令行 / MCP / 中转站）· 扫码收发页
                        │  命令 + 事件
                        ▼
  Rust 引擎：身份 · 发现 · 连接阶梯 · 传输 · 同步 · 插件宿主
                        │  QUIC：设备密钥双向认证，端到端加密
                        ▼
    本地直连 → 公网直连 → 端口映射 → 打洞 → 自建中转站
```

- **无账号的身份**：每个人有一把个人密钥，给自己的每台设备签发设备证书；添加联系人就是存下对方的公钥，从此认得他的所有设备
  （[身份模型](book/src/identity/model.md)）。
- **连接阶梯**：连接一台设备时并发尝试本地直连、公网 IPv6 / IPv4、路由器端口映射与打洞，都不通时才经中转站
  （[连接阶梯](book/src/network/ladder.md)）。
- **自建中转站**：任何一台常开设备都能一键成为中转站；它不终止两端的加密连接，只看得到密文
  （[中转站的数据边界](book/src/network/relay.md)）。
- **一切皆命令**：界面、命令行、MCP 与插件调用同一个命令注册表，测试也打在这一层（[命令与事件](book/src/architecture/commands.md)）。

## 文档

| 主题 | 位置 |
|---|---|
| 用户、场景与功能范围 | [book/src/product/](book/src/product/) |
| 身份模型：人、设备、联系人、空间 | [book/src/identity/](book/src/identity/) |
| 网络：发现、近场交付、连接阶梯、中转站、远程访问 | [book/src/network/](book/src/network/) |
| 架构：引擎、外壳、命令与存储 | [book/src/architecture/](book/src/architecture/) |
| 安全与威胁模型 | [book/src/security.md](book/src/security.md) |
| 待细化的问题 | [book/src/open-questions.md](book/src/open-questions.md) |
| 术语表 | [GLOSSARY.md](GLOSSARY.md) |
| 架构决定记录（ADR）| [docs/adr/](docs/adr/) |

完整目录见 [book/src/SUMMARY.md](book/src/SUMMARY.md)。设计书可以用 [mdBook](https://rust-lang.github.io/mdBook/) 在本地预览：

```bash
mdbook serve book
```

## 路线图

| 里程碑 | 内容 |
|---|---|
| M0 | 地基：关键技术实验、命令注册表骨架、CI 门禁 |
| M1 | 附近互传（桌面版、命令行）|
| M2 | Android 版 |
| M3 | 近场免安装：扫码收发与系统直传 |
| M4 | 我的设备、联系人、跨网连接与中转站 |
| M5 | 共享空间与常开设备 |
| M6 | 远程访问 |
| M7 | iOS 版 |
| M8 | 自动化：MCP、Skill、插件 |
| M9 | 发布 |

每个里程碑以「真人在真设备上完成场景」加自动化测试为完成标准，详见[路线图](book/src/roadmap.md)。

## 参与贡献

项目处于设计阶段，最需要的是对设计的讨论与质疑：

- 对设计有疑问或建议，请在 [Issues](https://github.com/AA4C-ORG/AA4C/issues) 中提出；[待细化的问题](book/src/open-questions.md)是很好的起点。
- 修改文档前，请先阅读 [AGENTS.md](AGENTS.md)（人与 AI agent 共用的协作约定）、[GLOSSARY.md](GLOSSARY.md) 与相关 [ADR](docs/adr/)。
- 提交信息使用英文 [Conventional Commits](https://www.conventionalcommits.org/)；行为变化同步修改设计书，难以回头的决定写成 ADR。

## 安全

请不要为安全漏洞公开提 issue。报告方式与响应承诺见 [SECURITY.md](SECURITY.md)。

## 许可证

许可证尚未选定，将在合入第一行产品代码之前确定（倾向 `MIT OR Apache-2.0`，见[待细化的问题](book/src/open-questions.md)第 18 条）。

## 联系

- 用户支持：support@aa4c.com
- 开发与安全：dev@aa4c.com
