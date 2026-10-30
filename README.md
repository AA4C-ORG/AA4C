# AA4C（AA连接）

**把文件 AA 给身边和远方的人。不用账号，不经网盘；优先直连，必要时只经你选择的自建中转站转发密文。**

> 状态：**设计阶段**。这是 AA4C 的第二代，从零重建；目前只有设计文档，还没有代码。
> 第一代（v0.1–v0.8）在 [HuoTaoCN/AA4C](https://github.com/HuoTaoCN/AA4C)，安装包仍可下载，但不会与新版互通。

## 它解决什么

- 朋友聚会、同事之间：有 AA4C 就加密直连；对方没安装也能扫码接收。没有共同 Wi-Fi 时，平台支持的话可临时建立 Wi-Fi Direct
  或热点，或交给系统分享（AirDrop、Quick Share 等）。
- 手机和电脑之间：照片、文件、一段文字、一个链接，随手互传。
- 远方的家人朋友：加一次联系人，以后不在一个网络也能直接发大文件。
- 一家人、一个小团队：共享文件夹，全家手机相册自动汇到家里的电脑或 NAS。
- 自己的几台电脑和 NAS：在单位或外面 SSH、远程桌面到家里的设备（直连或经自建中转站可达时）。

## 文档

| 想了解 | 看这里 |
|---|---|
| 用户、场景、核心功能 | [book/src/product/](book/src/product/) |
| 人 · 设备 · 联系人 · 空间（不要账号的身份模型）| [book/src/identity/model.md](book/src/identity/model.md) |
| 设备怎么跨网络连上 | [book/src/network/](book/src/network/) |
| 架构：一个引擎，四个外壳 | [book/src/architecture/](book/src/architecture/) |
| 为什么断代重建、什么可以移植 | [book/src/rebuild.md](book/src/rebuild.md) |
| 还没想清楚、需要讨论的问题 | [book/src/open-questions.md](book/src/open-questions.md) |
| 术语 | [GLOSSARY.md](GLOSSARY.md) |
| 已做的决定及理由 | [docs/adr/](docs/adr/) |
| 参与开发（人和 AI agent）| [AGENTS.md](AGENTS.md) |

完整目录见 [book/src/SUMMARY.md](book/src/SUMMARY.md)。

## 联系

- 网站：[aa4c.com](https://aa4c.com)
- 用户支持：support@aa4c.com
- 开发与安全问题：dev@aa4c.com（安全漏洞请勿公开提 issue，见 [SECURITY.md](SECURITY.md)）
