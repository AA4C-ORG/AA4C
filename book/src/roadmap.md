# 路线图

按**用户场景**切里程碑，每一步都在图形界面里交付、都能演示；完成标准是「真人用真设备做到了这件事」，并有对应的自动化测试守住。

| | 用户能做到什么 | 交付在 | 完成标准 |
|---|---|---|---|
| **M0 地基** | — | — | 见下 |
| **M1 附近互传** | 同一 WiFi 里看到身边的人，发文件、文件夹、文字；附近的人点接收即可 | 桌面版 + `aa4c` | 两台真机局域网互发；测试台覆盖自动接收、确认、拒绝、冷却、续传 |
| **M2 手机版** | 手机 ↔ 电脑互传；系统分享菜单「用 AA4C 发送」；相册选图 | Android | 真机：从相册分享到电脑、电脑发到手机 |
| **M3 近场免安装** | 对方没装 App：共同网络里扫码收发；没有共同路由器时，按平台能力建立本地链路或使用系统直传 | 桌面版 + Android | iOS Safari、Android Chrome、微信内置浏览器在共同局域网都能收和传；Android 真机至少跑通一种无路由器本地链路；能力不可用、取消和权限拒绝都如实显示 |
| **M4 我的设备与联系人** | 新设备扫码加入「我」；当面或邀请链接加联系人；一次性发送链接（接收方需安装）；跨网直连 / IPv6 / 端口映射 / 打洞；一键中转站与社交中转；连不上时给出原因和出路；agent 经 MCP 在我的设备之间直接收发 | 全部 | netsim 下「家 / 单位 / 手机」与「双方对称 NAT」场景全绿；真机清单走通；开始统计各档成功率；链接离线/过期/地址失效都给出真实原因；agent 经 MCP 把文件发到我的另一台设备，那台设备上的 agent 收到后接着处理，发给未授予的联系人被拒 |
| **M5 共享空间与常开设备** | 建家庭 / 团队空间、共享文件夹、手机相册自动备份到 NAS；在手机或电脑上远程管理 NAS | 全部 + `aa4c` | 一家三口场景真机走通：两部手机的照片汇到 NAS，三人都能浏览 |
| **M6 远程访问** | 在单位或外面 `ssh` 到自己的 NAS（直连或经自建中转站可达时）；用 SubnetDesk 或系统远程桌面控制自己的电脑 | 桌面版 + `aa4c` + Android | 见[远程访问](network/remote-access.md)的「M6 完成标准」|
| **M7 iOS** | iPhone 加入以上全部 | iOS | 同 M2–M6 的 iOS 部分（远程访问在 iOS 上只作前台发起端）|
| **M8 Skill 与插件** | AA4C Skill、Wasm 插件（归档规则）| `skills/aa4c` + 各外壳「更多」| skill 校验通过；恶意插件与越权意图全部被拒 |
| **M9 发布** | 安装包、应用商店、Docker 镜像、aa4c.com 上线 | — | 一条命令或一次点击装好；NAS 上一个容器跑起中转站 |

每个里程碑开工前，[待细化的问题](open-questions.md)里期限是它的条目必须已有结论（改了对应章节或写了 ADR）。

M6 不需要新的连接层：M4 的连接阶梯、中转站与 M5 的常开设备就是它的全部前提
（[ADR 0017](../../docs/adr/0017-remote-access-by-service-forwarding.md)）。

## M0 地基

1. 仓库骨架：本书、`AGENTS.md`、`GLOSSARY.md`、ADR（已完成）；Cargo 工作区与各 crate 空壳、`[workspace.lints]`、`xtask layers/deps`、三平台 CI。
2. 四个实验，各自一个最小可运行程序，结论写回对应 ADR：
   - **原始公钥 QUIC**：两端用 Ed25519 原始公钥完成 QUIC 握手并互相校验指纹，同时定下 rustls 的 crypto provider
     （[ADR 0005](../../docs/adr/0005-raw-public-keys.md)）。
   - **UniFFI + Android**：Rust 引擎在 Android 模拟器上跑起来，Kotlin 薄壳调用一个命令并收到事件（[ADR 0001](../../docs/adr/0001-rust-engine-native-mobile-shells.md)）。
   - **桌面界面**：首页（附近与联系人）分别用 egui 和 Slint 做原型，离屏截图对比，按[桌面版](surfaces/desktop.md)的标准定案。
   - **默认信标隐私**：用发现密钥做最小帧实验，抓包证明信标载荷没有稳定公开身份（包括真实条目数），测出 100 个联系人时的
     填充档位、大小与发现延迟上限（[ADR 0015](../../docs/adr/0015-private-nearby-discovery.md)）。
3. 三项选型比较，不做原型，出结论即可：redb 还是 fjall（[ADR 0006](../../docs/adr/0006-redb-not-sqlite.md)）；MCP 根目录用 `cap-std`
   还是各平台 `openat` 等价方式（[MCP 与 AI agent](surfaces/agents.md)）；首份依赖清单（`cargo xtask deps`）。
4. 命令注册表、`CallerContext` 与事件流的骨架；`aa4c commands --json` 能列出命令描述；MCP 的目标/路径/字节预算越权测试先于真实发送。
5. 自动门禁骨架：`xtask layers/deps`、文档构建与链接检查、skill 校验、三平台 CI。
6. 待定的工程决定：旧仓库处理（见[待细化的问题](open-questions.md)）。许可证已定为 Apache-2.0（[ADR 0019](../../docs/adr/0019-apache-2-license.md)）。

M0 的全部任务以本节为准，其他章节写到「M0」时都应能在这里找到。近场平台能力矩阵不进 M0，放到 M3 开工前
（[近场交付](network/proximity.md)）。

## 第一代的状态

第一代停在 v0.8.x，不再开发新功能，安装包仍可下载；第二代不与它互通（[ADR 0014](../../docs/adr/0014-clean-break-from-v1.md)）。
