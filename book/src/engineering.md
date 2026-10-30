# 工程规范

写给写代码的人和 AI agent。规则来源：storytold/photocraft 的工程纪律、mattpocock/skills 的仓库与文档规范、第一代踩过的坑。

## 文档与语言

- **`GLOSSARY.md`** 是统一语言：代码、测试名、提交信息、文档里的领域概念一律用它定义的词，不用它列为「避免」的同义词。
  概念不在表里，要么是在发明项目不用的语言（重新考虑），要么是真缺口（补进表里）。
- **`docs/adr/`**：只为「难回头 + 不看上下文会觉得奇怪 + 有真实取舍」的决定写 ADR。与 ADR 冲突时明说，写新 ADR，不要悄悄违背。
- **本书（`book/`）**：行为变了就改对应章节；`mdbook build book` 在 CI 里检查。
- **不写交接文档**：进度在 GitHub Issues，决定在 ADR，语言在 GLOSSARY。第一代 115 KB 的 HANDOFF 是反面教材。
- 写给 agent 的文档（AGENTS.md、SKILL.md）遵循「删掉一切删了也不改变行为的句子」：越改越短。
- 文档语言：设计书与用户文档中文优先；代码标识符、提交信息用英文。

## 代码硬规则

- **永不崩溃**：非测试代码禁止 `unwrap`、`expect`、`panic!`、`unreachable!`、`todo!`、`unimplemented!`（clippy 强制，测试中允许）。
  返回错误并用 `?` 传播；确属不可能失败的字面量，用 `#[allow]` 并写明理由。
- **不写 `unsafe`**：工作区 `unsafe_code = "forbid"`；唯一例外是 `ffi/aa4c-mobile`（UniFFI 生成代码）。
- **输入都是敌意的**：用 `get()` 而不是 `[i]`，用 `checked_*`/`saturating_*`，限制递归深度与内存分配。
- **锁中毒与线程 join 结果**都要处理，不级联崩溃。
- **一切皆命令**：新的用户可见行为 = 一个新命令（id、summary、参数/结果 Schema、能力级别、副作用、路径访问、实现、测试）。
- **分层**：新 crate 必须登记到 `xtask` 的分层表；低层不依赖高层。
- **深模块**：小接口、大实现；「一个适配器是假接缝，两个才是真接缝」，不为只有一种实现的东西造 trait。
- **依赖预算**见[分层与 crate](architecture/crates.md)。

## 测试

- **测试打在命令注册表上**：`engine.run("send.files", json!({…}))`，与命令行、MCP、界面走同一条路。测试描述行为，不测实现细节；
  期望值来自独立的真值（已知字面量、规格），不是把实现再算一遍。
- **TDD 竖切**：一个接缝、一个失败的测试、一个最小实现，循环。动手前先约定要测的接缝。
- **多人多设备测试台**：一个进程里起「我（3 台）+ 爸妈（2 台）+ 同事（1 台）」，各自独立数据目录，走真实 QUIC 与回环网络。
- **netsim**：在测试台上模拟不同网络（屏蔽直连、对称 NAT、UDP 黑洞、丢包、地址变化），覆盖连接阶梯的每一档与每一种失败原因。
- **panic_hunt**：对每个命令喂对抗性参数，必须全部返回错误而不崩溃。
- **policy_hunt**：用伪造 `CallerContext`、越界路径、未授权接收目标、超字节预算和动态关系变化轰炸命令；UI、MCP、插件入口结果一致。
- **fuzz**：协议解码、路径净化、证书解析、插件 manifest。
- **等待用墙钟预算**，不用固定循环次数（第一代教训：不匹配的事件会白白吃掉循环配额）。
- **修崩溃必须附回归测试**，并证明修复前它会失败。
- **不在自动化测试里碰真实路由器**（UPnP），用假后端；真机验证写成清单由人执行。

## 界面验证

- 界面改动必须附离屏截图，人眼看过。构建通过不等于界面正确——第一代连续七个版本没做过界面走查。
- 自动化检查：界面上每个可点击动作都能在 `aa4c commands` 里找到对应命令。

## 提交前

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask layers
cargo xtask deps
cargo xtask commands                                    # 命令描述、CLI 与 MCP 映射一致
cargo xtask docs                                        # mdBook、内部链接、ADR 索引
cargo xtask skills                                      # SKILL.md 与 agents/openai.yaml
cargo test -p aa4c-engine --test panic_hunt -- --ignored   # 改了命令时
cargo test -p aa4c-engine --test policy_hunt -- --ignored  # 改了授权时
```

## CI

- 三平台（Linux、macOS、Windows）测试 + lint + 依赖审计 + 分层/依赖预算 + 命令映射 + 文档/skill 校验 + Android 构建 + 引擎绑定冒烟测试；iOS 自 M7 起。
- **`continue-on-error` 的任务必须有人定期看**：第一代的 Android 哨兵红了一个月没人发现，因为它永远不会让 CI 变红。
- 下载外部固件/工具一律 `curl --retry 5 --retry-all-errors`（单用 `--retry` 不重试证书类错误）。
- CI 红了当场修，不往上堆新改动。第一代两次「CI 红了一周/一个月没人发现」，期间的问题是叠在一起的。

## 提交与协作

- 提交信息英文，Conventional Commits，一个提交一件事；提交身份 `AA4C <dev@aa4c.com>`。
- 每完成一个可独立验证的阶段就合入、推送、等 CI 出结果再继续。
- 远端仓库与 issue 标签就绪后，先运行 mattpocock/skills 的 `setup-matt-pocock-skills`；不要凭空假设 tracker 与标签。
  日常流程：`/grill-with-docs`（对齐需求、维护 GLOSSARY 与 ADR）→ `/to-spec` → `/to-tickets` →
  `/implement`（内含 `/tdd`）→ `/code-review`。规格写模块与测试接缝，不写很快失效的实现文件清单。
- 多个 agent 并行时：各用自己的 `CARGO_TARGET_DIR`；新命令放新模块，避免多人同时改一个共享文件。
