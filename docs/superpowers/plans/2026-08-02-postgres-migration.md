# 数据库迁移计划 (已作废 — 被 SQLite 取代)

> **状态**: **Superseded — 计划已作废**

## 作废说明

历史沿革：本系统由钢管行业系统重构而来，重构前曾计划将数据库迁移至 PostgreSQL，该计划已**整体作废**，不包含任何可执行任务。

重构后数据库定为 **SQLite3**（连接串 `sqlite://data/erp.db?mode=rwc`，sqlx 0.8 `sqlite` feature），单文件、零外部数据库依赖：

- 原计划中的驱动切换、`$N` 占位符重排、`BIGSERIAL`/`TIMESTAMPTZ` 类型转换、本地测试库、`pg-dev.sh` 等任务**全部不再适用**。
- 37 个遗留迁移文件将由代码阶段**重写为 SQLite 语法**，并删除钢管行业专属表（管材、标签、质检证书、参考数据等；完整清单见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`）。
- 测试基建沿用 SQLite 文件/内存库，无需外部数据库服务器。

如需追溯原计划，请查看 Git 历史中本文件删除前的内容。
