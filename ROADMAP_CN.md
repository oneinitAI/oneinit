# OneInit 路线图 / Roadmap（中文版）

> 项目目标与发展方向。这里标注了已发布的能力与下一步计划，
> 贡献者可以在这里找到「在哪里贡献价值」。

## ✅ 已发布 / Released

**v0.1.0-beta 系列**（当前：`v0.1.0-beta.2`）

- ✅ 一键初始化：内置配方（Python/Node/Go/Java）+ 自动镜像源（pip 清华 / npm 淘宝）
- ✅ 社区配方系统：YAML 配方 + 注册表（oneinit-recipes）+ 多注册表订阅 + 3 层解析
- ✅ 环境采集与迁移：capture（7 种语言检测器）/ export / import
- ✅ 干净卸载：SQLite 清单追踪，卸载彻底回滚
- ✅ 交互式 TUI
- ✅ AI 友好：`--json` 结构化输出 + Skill 集成
- ✅ 供应链安全：SHA256 校验和验证 / `--allow-exec` 默认拒绝 / 注册表 Ed25519 签名
- ✅ 团队环境同步（oneinit team）：共享开发环境自动检测同步 + 可选签名
- ✅ 环境可视化（oneinit viz）：ASCII 树 / HTML(SVG) 报告 / Issue 快照
- ✅ 官网（中英双语）+ /changelog + npm 发行

## 🔜 下一步 / Next（v0.2.0 目标）

按贡献价值排序，🟢 标记适合新手的入口：

| 事项 | 说明 | 状态 |
|------|------|------|
| `oneinit sync` 应用镜像源 | 本地 oneinit.yaml 的 mirrors 目前只打日志，复用团队同步的 apply_mirrors | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| `oneinit cache clean` | 缓存/临时目录清理命令（temp/ + 过期缓存） | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| TUI 团队环境状态 | TUI 显示团队同步状态 / 提供同步入口 | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| 环境快照 dotfiles | capture/export 支持 dotfile 收集与恢复（设计文档已就绪） | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| 工具版本选择 | `oneinit install python@3.11` 显式版本选择（当前主要用配方默认版） | 规划中 |
| `oneinit outdated` | 检查已装工具是否有新版本 | 规划中 |
| 自更新 | `oneinit self-update`（校验 SHA256SUMS 后自动升级） | 规划中 |
| TUI 搜索增强 | 可用列表搜索过滤 | 规划中 |

## 🔭 远期 / Later

- **Windows 原生安装器**（MSI）与更好的系统集成
- **配方生态扩展**：更多配方、配方版本管理、赞助者配方
- **团队环境可视化官网面板**：在官网展示团队环境状态
- **v1.0.0 稳定版**：API 稳定、文档完备、安全审计完成

## 🧭 如何影响路线图

- **想要某个功能**：开 `feature request` Issue，说明场景与价值，参与讨论
- **想立刻贡献**：从 🟢 Good First Issues 开始 —— 它们范围明确、有清晰验收标准、
  能让你获得"快速胜利"的成就感
- **路线图本身**：有意见或更好的优先级建议，欢迎在 Issues 里讨论

---

*路线图会随项目演进持续更新。当前状态同步于 v0.1.0-beta.2。*
