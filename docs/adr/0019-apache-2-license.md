# 0019 许可证用 Apache-2.0

状态：已接受（2026-10-30）

许可证一旦有外部贡献就很难再改。候选是第一代用过的 Apache-2.0，和 Rust 生态常见的 `MIT OR Apache-2.0` 双许可。

决定：沿用第一代的 **Apache-2.0**，版权行为「Copyright 2026 HuoTao and AA4C contributors」，全文见仓库根目录的 `LICENSE`。

理由：与第一代一致，移植第一代的纯逻辑模块与测试时只需保留原有版权声明，不必再逐个征得贡献者同意；
Apache-2.0 自带专利授权条款，对一个处理网络与加密的项目比单独的 MIT 更稳妥。

后果：新增依赖的许可证必须与 Apache-2.0 兼容，在 `cargo xtask deps` 的清单里检查；AGPL 等强著佐权代码（例如 SubnetDesk）
只能以独立进程配合，不能链接进 AA4C（ADR 0017）。
