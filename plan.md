# OneInit 开发计划

## 开发原则
1. **一次一任务，一次一提交** -- 使用 `gh` CLI 每次修改都提交
2. **任务化** -- 每次要做什么先看本计划，有规划地推进
3. **记录** -- 重要信息记录在 `Important.md`
4. **代码兼容性** -- 源码中禁止使用 emoji，用 ASCII 标记代替（如 `[OK]` `[ERROR]`）
5. **架构一致性** -- 新功能必须复用现有 CoreError/Result 统一错误类型，不引入独立的 per-module error 枚举

---

## Phase 1: MVP（已完成）

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 1 | 项目初始化 + CLI 骨架 | 完成 | Git 仓库、Rust 项目、clap 命令骨架、--json 开关 |
| 2 | 核心引擎：下载器 | 完成 | 异步下载 + SHA256 校验 + zip/tar.gz 解压 |
| 3 | 核心引擎：清单系统 | 完成 | SQLite 存储安装记录，WAL 模式，支持回滚 |
| 4 | 核心引擎：PATH 管理 | 完成 | Windows 注册表 + Unix shell 配置文件 |
| 5 | 核心引擎：配置生成 | 完成 | pip 清华源、npm/yarn 淘宝镜像 |
| 6 | 配方系统设计 | 完成 | 定义配方格式，实现首个配方（Python 3.11.9） |
| 7 | 集成测试：Python | 完成 | `oneinit install python3.11` 全流程验收通过 |

## Phase 2: 生态扩展

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 8 | 包仓库扩展 | 待做 | Java, Go, Rust, MySQL, Redis（随社区配方机制一起） |
| 9 | `init` 命令完整实现 | 完成 | 预置套装（python/ai/frontend/full） |
| 10 | `sync` 命令完整实现 | 完成 | 读取 `oneinit.yaml` 批量同步 |
| 11 | 社区配方机制 | 完成 | 声明式 YAML 配方 + `verify` 命令 + 安装安全提醒 + `~/.oneinit/recipes/` |

## Phase 3: 体验优化

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 12 | TUI 界面 | 完成 | ratatui + crossterm 异步事件循环 + 双面板菜单 |
| 13 | 企业功能 | 待做 | 私有配方仓库、离线安装包（与 export/import 合并） |
| 14 | TUI 功能完善 + 文档清理 | 完成 | 移除源码 emoji（兼容性）、完善 plan.md/Important.md |

> 注：原 GUI 桌面应用（Tauri）计划已取消，完全转向 TUI 方向。

---

## Phase 4: 数据采集与迁移（规划中）

> 设计文档：`数据的采集与迁移.md`
> 这是一个极其复杂的功能集，分为两个独立子系统：环境捕获（capture）和数据迁移（export/import）。
> 每个子系统需要 3-5 个提交，总计约 15+ 个新文件。

### 4A. 环境捕获（`oneinit capture`）

**核心目标**：非侵入式扫描当前机器已安装的开发环境，生成 `oneinit.yaml` 配方。

| # | 任务 | 状态 | 说明 | 新增依赖 |
|---|------|------|------|----------|
| 15 | 跨平台适配层 + 核心数据结构 | 完成 | EnvDetector trait + DetectorScheduler + find_command + 数据结构 | 零 |
| 16 | Python 检测器 | 完成 | python3/python.exe + pip 镜像 + 全局包列表 | 零（同步 trait，无 async-trait） |
| 17 | Node.js + Git 检测器 | 完成 | node/npm 版本 + npm registry + 全局包; git 版本 + user config | 零 |
| 18 | `capture` 命令 + 配方生成 | 完成 | `oneinit capture [-o file]` -> EnvironmentSnapshot -> YAML | 零 |
| 19 | TUI capture 交互界面 | 完成 | 按 `c` 触发检测，Capture 屏幕显示结果 | 零 |

> 扩展检测器：Rust/Go/Java/Docker 已添加（共 7 个检测器）。find_command 增强为多策略（where/which -> PATH 遍历 -> exe 扩展名）。支持用户自定义检测器（scan_config.yaml）。

### 4B. 数据迁移（`oneinit export` / `import`）

**核心目标**：将完整开发环境打包为 `.tar.gz`，在新机器上一键恢复。

| # | 任务 | 状态 | 说明 | 新增依赖 |
|---|------|------|------|----------|
| 20 | 迁移清单结构 `migration/manifest.rs` | 完成 | manifest.json 结构（CacheEntry/PackageListEntry/ManifestMetadata） | 零 |
| 21 | 导出打包器 `migration/packer.rs` | 完成 | 扫描环境+YAML+可选envs打包+manifest.json+tar.gz | 零（复用 flate2/tar/sha2/uuid） |
| 22 | 导入解包器 `migration/unpacker.rs` | 完成 | 解压+SHA256校验+恢复配方/envs+dry_run 预览 | 零 |
| 23 | `export` / `import` CLI 命令 | 完成 | `oneinit export [-o] [--include-envs]` / `oneinit import [--dry-run] [--force]` | 零 |
| 24 | TUI 迁移界面 | 待做 | 导出预览、导入 dry-run 展示 | 无 |

> Phase 4A+4B 核心功能已完成。capture 支持 7 种语言检测 + 自定义检测器；export/import 完整流程测试通过（6 环境 + 129 全局包 + 可选 envs 缓存打包）。

## Phase 5: AI 集成

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 25 | ZCode AI Skill | 完成 | `.agents/skills/oneinit/SKILL.md`，覆盖全部 12 个 CLI 命令，含 --json 最佳实践、社区配方格式、模板变量速查 |

### 架构适配要求（关键！）

设计文档中的代码示例**不能直接使用**，需要适配：

1. **错误类型**：文档提出 `DetectorError`/`GeneratorError`/`ExportError`/`ImportError` 等独立枚举 -> **必须统一到 `CoreError`**（新增 `Capture(String)` / `Migration(String)` 变体）
2. **async-trait**：文档使用 `#[async_trait]` -> Edition 2024 原生 async trait 可能更好，需验证
3. **模块路径**：文档假设 `oneinit-core` 是独立 crate -> 实际是 `src/core/` 子目录，路径需调整
4. **依赖冲突**：文档写的 `winreg = "0.52"` / `dirs = "5"` -> 实际已是 `winreg = "0.55"` / `dirs = "6"`，版本需对齐
5. **tempfile 安全**：`tempfile::tempdir()` 在 Windows 上的行为需测试，可能需用现有的 `temp_dir()`
6. **regex 版本**：文档未指定版本 -> 应使用 `regex = "1"`
7. **zstd 可选**：文档同时提出 gzip 和 zstd -> MVP 先只做 gzip（flate2 已有），zstd 后续扩展

### 新增依赖清单（Phase 4 总计）

```toml
# 必需
async-trait = "0.1"     # 检测器 trait（如果不用原生 async trait）
walkdir = "2"           # 递归遍历目录（打包/解包）
tempfile = "3"          # 安全临时目录
gethostname = "0.4"     # 导出时记录主机名
regex = "1"             # 版本号解析（如 "Python 3.11.9"）

# 可选（后续阶段）
# zstd = "0.13"         # zstd 压缩（MVP 先用 gzip）
# sysinfo = "0.30"      # 服务状态检测（MySQL/Redis）
```

---

## 里程碑

- **MVP 达成**（Phase 1）：`oneinit install python3.11` 全流程可用
- **生态扩展**（Phase 2）：init/sync 命令、预置套装、社区配方
- **体验优化**（Phase 3）：TUI 界面、emoji 清理、文档完善
- **数据迁移**（Phase 4 规划中）：环境捕获 + 离线迁移（~15 个新文件，~2000 行新代码）
