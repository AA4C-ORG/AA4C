# AGENTS.md

AA4C 是一个让人与人之间、设备与设备之间直接传文件，并让自己的设备彼此连得上的开源软件：无账号、无官方服务器；
优先直连，用户可自建中转站。对 AI agent 友好：命令行与 MCP 覆盖全部功能，在我的设备之间 agent 可以直接动手
（[ADR 0018](docs/adr/0018-agents-act-directly-across-my-devices.md)）。
一个 Rust 引擎，配桌面版、手机版、无界面版 `aa4c`、扫码收发页四个外壳。

**当前阶段：设计。** 还没有代码。在 [book/src/roadmap.md](book/src/roadmap.md) 的 M0 开始之前，
不要写产品代码；改动只针对文档。

## 先读

1. [GLOSSARY.md](GLOSSARY.md)：用这里的词，别用它列为「避免」的同义词。
2. [book/src/SUMMARY.md](book/src/SUMMARY.md)：从目录找到与任务相关的章节。
3. [docs/adr/](docs/adr/)：动到某个领域前，先读该领域的 ADR。要推翻一条 ADR，明说并写新的 ADR，不要悄悄违背。

## 硬规则

- **产品边界**：不做下载器、AI/知识库、聊天、云盘、账号体系、社区（[ADR 0009](docs/adr/0009-no-downloader-no-ai.md)）。
  远程访问只做服务转发：引擎里不做截屏、输入注入与音视频编解码，也不做虚拟网卡（[ADR 0017](docs/adr/0017-remote-access-by-service-forwarding.md)）。
- **一切皆命令**：每个用户可见的行为都是引擎命令注册表里的一个命令；桌面、手机、CLI、MCP、插件都只调命令
  （[book/src/architecture/commands.md](book/src/architecture/commands.md)）。
- **语言**：业务逻辑只用 Rust。例外只有两处：手机薄壳（Kotlin/Swift，只画界面、转调引擎并适配系统能力，不写选择策略）与扫码收发页
  （纯 HTML，不用 JavaScript）（[ADR 0001](docs/adr/0001-rust-engine-native-mobile-shells.md)）。
- **分层**：低层 crate 不依赖高层；UI 库只出现在最上层（[book/src/architecture/crates.md](book/src/architecture/crates.md)）。
- **永不崩溃**：非测试代码不用 `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`；一切来自网络、文件、参数的值都当敌意输入。
- **不写 `unsafe`**：唯一例外是隔离的 FFI crate。
- **依赖预算**：`aa4c` 无界面版 ≤ 130 个唯一 crate；新增依赖要写明理由、是否处理不可信输入、有无 C 代码。
- **信任变更只能由人做**：加联系人、加入设备、改空间成员、吊销设备属于 `trust` 级命令，MCP 默认不暴露
  （[ADR 0011](docs/adr/0011-trust-changes-are-human-only.md)）。
- **测试打在命令注册表上**，与 CLI/MCP/界面走同一条路（[book/src/engineering.md](book/src/engineering.md)）。

## Agent 工作流

- 领域语言在 `GLOSSARY.md`，难回头的决定在 `docs/adr/`；不要在 issue、skill 或代码注释里再造第二套定义。
- 规格优先写用户故事、模块接缝与测试接缝，不把临时文件路径当长期设计。最高测试接缝是命令注册表。
- 进入实现阶段后，用小的纵向切片推进：先约定命令与可观察结果，再写失败测试，再写最小实现，最后代码审查。
- `skills/aa4c/SKILL.md` 是「怎么使用已安装 AA4C」的产品 skill，不是开发说明；命令参数始终从 `aa4c commands --json` 发现。

## Agent skills

### Issue tracker

issue 放在 GitHub `AA4C-ORG/AA4C` 的 Issues 里。见 `docs/agents/issue-tracker.md`。

### Triage labels

默认的五个分诊标签，标签名与角色同名。见 `docs/agents/triage-labels.md`。

### Domain docs

单一上下文：`GLOSSARY.md`、`docs/adr/` 与设计书 `book/src/`。见 `docs/agents/domain.md`。

## 提交与发布

- 提交信息用英文，Conventional Commits（`feat:` / `fix:` / `docs:` / `refactor:` / `test:` / `chore:`），一个提交一件事。
- 提交身份 `AA4C <dev@aa4c.com>`（本机已按目录自动设置），在 GitHub 上对应 AA4C-ORG。
  提交信息里不加 `Co-Authored-By` 等 AI 署名 trailer：仓库的贡献者只显示 AA4C-ORG。
- 任何 GitHub 上的创建、推送、转移、发布，先征得维护者同意。
- 改了行为就同步改对应的 book 章节；新术语进 GLOSSARY；难回头的决定写 ADR（格式见 [docs/adr/README.md](docs/adr/README.md)）。
