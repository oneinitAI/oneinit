# OneInit — 面向 AI 的项目 Wiki

> 本文档是给 AI 助手（以及想要快速上手的开发者）看的项目全貌指南。
> 它描述代码结构、数据流、核心概念、约定与常见开发任务。
> 若要了解用户侧用法，见 `README.md` / `README_CN.md`；若要了解配方生态，见 `.agents/skills/oneinit/SKILL.md` 与 `oneinit-recipes/`。

---

## 1. 项目概述

OneInit 是一个 **AI-first 的开发环境初始化 CLI**：用一条命令完成"下载安装工具、自动配置镜像源、写入 PATH、SQLite 记录清单以便干净回滚"。全部安装在用户空间（`~/.oneinit/`），零 sudo。

- **语言**：Rust（Edition 2024）
- **版本**：`0.5.0-beta.1`（预览版）
- **许可证**：GPL-3.0
- **仓库**：`oneinitAI/oneinit`（核心代码）；`oneinitAI/oneinit-recipes`（社区配方注册表）
- **核心卖点**：
  - 所有命令支持 `--json` 结构化输出，供 AI 解析后自主决策
  - 每次运行自动静默刷新配方索引（缓存缺失或 >24h 时）
  - 内置→本地社区→远程注册表→动态配方，四级配方解析
  - `--dry-run` 用统一的 Operation Plan 预览，与真实执行走同一路径

### 生态构成

| 目录/仓库 | 作用 |
|---|---|
| `oneinitAI/oneinit`（本仓库） | Rust CLI 核心 |
| `oneinitAI/oneinit-recipes` | 社区配方仓库：`INDEX.json` + `recipes/<name>/<version>.yaml`，CI 自动校验 |
| `oneinitAI/oneinit-team-env` | 团队环境模板（`team.yaml` + 可选 Ed25519 签名工作流） |
| `www/` | Next.js 官网（非核心） |
| `npm/` | npm 安装包装器（postinstall 下载二进制） |

---

## 2. 目录结构

```
F:\oneinit
├── src/
│   ├── main.rs                 # 入口：clap 定义全部命令 + async 分发 + 自动更新/团队检测
│   ├── cli/mod.rs              # 各子命令的 handler（2,269 行，项目最大文件）
│   ├── core/                   # 核心引擎
│   │   ├── mod.rs              # 目录助手(data_dir/envs_dir/db_dir/...) + CoreError + Result
│   │   ├── recipe.rs           # 内置配方（python3.11/node20/go/java17）+ 安装/卸载执行器
│   │   ├── community_recipe.rs # 社区配方 YAML DTO + 加载/验证/模板渲染/安装
│   │   ├── operation.rs        # Operation 枚举 + OperationPlan + 摘要统计
│   │   ├── planner.rs          # 配方→OperationPlan（dry-run 与执行共用）
│   │   ├── downloader.rs       # 异步下载 + SHA256/512 校验 + zip/tar.gz 解压
│   │   ├── manifest.rs         # SQLite 安装清单（WAL 模式）
│   │   ├── path_mgr.rs         # 跨平台 PATH（Windows 注册表 / Unix shell 配置）
│   │   ├── config_gen.rs       # 镜像源配置文件生成（pip/npm/yarn）
│   │   ├── preset.rs           # 预置套装（python/ai/frontend/full）
│   │   ├── sync.rs             # oneinit.yaml / team.yaml 解析 + post_install
│   │   ├── registry.rs         # 社区注册表（INDEX.json 拉取/合并/签名校验/缓存）
│   │   ├── dynamic.rs          # 动态配方构建（python@3.11 等版本化家族）
│   │   ├── version.rs          # 版本解析（@latest/@lts/部分匹配）
│   │   ├── checksum.rs         # 动态配方校验和解析（官方源优先）
│   │   ├── cache_db.rs         # 版本/校验和 SQLite 缓存
│   │   ├── team.rs             # 团队环境同步（team.yaml + Ed25519 签名 TOFU）
│   │   ├── viz.rs              # 环境可视化（ASCII 树/HTML(SVG)/Issue 快照）
│   │   ├── doctor.rs           # 环境健康检查引擎
│   │   ├── self_update.rs      # 自更新（GitHub Release + SHA256SUMS 校验）
│   │   ├── capture/            # 环境捕获：detector.rs + python/node/git/rust/go/java/docker
│   │   └── migration/          # 数据迁移：manifest.rs + packer.rs + unpacker.rs
│   ├── output/mod.rs           # OutputFormatter（human/json 双模式 + error + HINT）
│   ├── security.rs             # 免责声明常量
│   ├── skill_mgr.rs            # 把 SKILL.md 安装到各 AI 助手目录
│   ├── tui2/                   # ratatui TUI（mod/backend/event/screens/state/app）
│   └── i18n.rs                 # 国际化占位（当前 pass-through）
├── .agents/skills/oneinit/     # 内置 AI Skill（SKILL.md + recipe-wizard/）
├── oneinit-recipes/            # 配方仓库的本地镜像/工作副本（INDEX.json 等）
├── www/                        # Next.js 官网
├── npm/                        # npm 包装器（scripts/install.js + bin/oneinit）
├── scripts/                    # install.js（npm postinstall）等
├── install.sh / dev.sh         # 一键安装脚本 / 本地开发环境配置脚本
├── plan.md                     # 开发计划（各 Phase 状态）
└── Important.md                # 重要笔记（结构/约定/依赖清单）
```

---

## 3. 技术栈与依赖

| crate | 版本 | 用途 |
|---|---|---|
| clap / clap_complete | 4.x | CLI 框架（derive 模式）+ shell 补全 |
| tokio / tokio-stream | 1.x | 异步运行时 + 事件流 |
| serde / serde_json / serde_yaml | 1.x | 序列化 / JSON 输出 / YAML 解析 |
| indicatif | 0.18 | 下载进度条 |
| thiserror | 2.x | 错误类型派生 |
| reqwest | 0.12 | HTTP 客户端（features = stream） |
| futures-util | 0.3 | Stream 支持 |
| sha2 | 0.10 | SHA256 |
| flate2 / tar / zip | — | 压缩与归档 |
| rusqlite | 0.32 (bundled) | SQLite（清单 + 缓存） |
| uuid | 1.x | manifest ID |
| dirs | 6.x | home 目录 |
| chrono | 0.4 | 时间戳 |
| ratatui / crossterm | 0.29 / 0.28 | TUI |
| ed25519-dalek | 2.x | 索引/团队配置签名校验 |
| winreg / winapi | 0.55 / 0.3 | Windows 注册表（仅 cfg(windows)） |

---

## 4. 入口与命令分发（src/main.rs）

- `Cli` 结构体（clap derive）定义了全局参数：
  - `--json`：结构化输出（AI 友好）
  - `-y/--yes`：跳过交互确认
  - `-v/--debug`：调试输出
- `main()` 流程：
  1. 解析 CLI，构造 `OutputFormatter`
  2. 高风险命令（install/sync/import/team add/team sync）先打印安全免责声明（`security.rs`）
  3. **自动刷新配方索引**（`maybe_auto_update`）：缓存缺失或 >24h 时静默拉取，失败不阻塞（`Update`/`Registry`/`Completions`/`Team`/`Sync`/`Capture`/`Export`/`Import`/`Freeze`/`Viz`/`SelfUpdate` 命令跳过）
  4. **团队环境自动检测**（`cli::maybe_team_sync`）：24h 间隔 + 内容哈希，变化时才同步，失败仅 `[WARN]`
  5. 分发到 `cli::run_*`

**命令全集**（cli 层实现函数见 `src/cli/mod.rs`）：

| 命令 | 实现 | 说明 |
|---|---|---|
| `init` | `run_init` | 预置套装 / `--project` 项目感知安装 |
| `install <pkg>` | `run_install` | 支持 `name@version`，`--allow-exec`/`--dry-run`/`--refresh`/`--no-checksum` |
| `uninstall` | `run_uninstall` | 先试内置回滚，再试社区回滚 |
| `list` | `run_list` | `--format table/csv`；`list versions <recipe>` |
| `info <pkg>` | `run_info` | 版本解析详情 |
| `search [kw]` | `run_search` | 四来源：builtin/community/remote/dynamic |
| `sync` | `run_sync` | 从 `oneinit.yaml` 批量同步 |
| `capture` | `run_capture` | 7 个检测器扫描环境 → YAML |
| `verify <file>` | `run_verify` | 校验社区配方 |
| `update` | `run_update` | 拉取全部注册表 INDEX.json |
| `registry add/remove/list` | `run_registry_*` | 多订阅管理 |
| `issue [kind]` | `run_issue` | 打开配方仓库 issue 表单 |
| `publish <file>` | `run_publish` | 验证 + 打印 git PR 发布步骤 |
| `export/import` | `run_export`/`run_import` | tar.gz 迁移 |
| `tui` | `tui2::run_tui` | 交互式界面 |
| `doctor` | `run_doctor` | 健康检查 |
| `freeze` | `run_freeze` | 导出已装工具为 oneinit.yaml |
| `self-update` | `run_self_update` | 自更新 |
| `completions <shell>` | 内联 | bash/zsh/powershell/fish/elvish |
| `skill install/list/status/uninstall` | `run_skill_*` | 管理 AI Skill |
| `team add/remove/status/sync` | `run_team_*` | 团队环境 |
| `viz` | `run_viz` | ASCII 树 / `--html` / `--issue` |

别名：`i`→install，`u`/`rm`→uninstall，`ls`→list，`up`→sync，`check`→doctor，`upgrade`→self-update。

---

## 5. 输出系统（src/output/mod.rs）

`OutputFormatter` 是**唯一**的业务输出通道，禁止直接 `println!` 业务数据（见 Important.md 约定）。

关键点：
- `output(human_text, json_data)`：human 模式打印文本；json 模式把 `json_data` 序列化为独立 JSON 文档。`Some(serde_json::Value::Null)` 表示"仅装饰行"，json 模式下被抑制。
- `error(&CoreError)`：统一错误输出，附 `suggestion()`（HINT / JSON `"suggestion"` 字段）。
- `begin_document(action)` / `end_document()`：把后续多条 output 缓冲为一个 `{action, count, items}` 文档（human 模式无操作）。
- `debug_line(msg)`：仅在 `--debug` 时输出。
- `auto_yes` / `debug` 两个公共字段由 `main()` 从全局参数注入。

---

## 6. 核心概念与数据流

### 6.1 配方（Recipe）

三种配方来源，统一抽象为可安装的对象：

1. **内置配方**（`core/recipe.rs`）：硬编码 `Recipe` 结构体。目前 4 个：
   - `python3.11` (3.11.9，Windows embeddable zip + get-pip + 清华源)
   - `node20` (20.18.1，npm npmmirror)
   - `go` (1.23.4)
   - `java17` (17.0.20+8，Temurin)
2. **社区配方**（`core/community_recipe.rs`）：声明式 YAML，存在 `~/.oneinit/recipes/*.yaml` 或从远程注册表拉取。字段：`name/version/description/platforms/post_install/depends/tags/maintainer/license`。
   - `platforms`: `windows/linux/darwin` 各含 `{url, sha256, install_type, install_args?, install_path, path_add}`
   - `install_type` 合法值：`zip_extract`、`tar_extract`、`exe_silent`、`msi_install`、`pkg_install`、`binary_copy`
   - 模板变量：`{{install_dir}}`、`{{user_home}}`、`{{mirror_pip}}`、`{{mirror_pip_host}}`、`{{mirror_npm}}`（`render_template`）
3. **动态配方**（`core/dynamic.rs` + `version.rs` + `checksum.rs`）：版本化家族（python/node/go/java/rust），如 `python@3.11`、`node@lts`。由代码实时构建 `CommunityRecipe`，URL 按版本参数化，校验和从官方源（nodejs.org SHASUMS256.txt、python.org SPDX SBOM、dl.google.com sidecar、Adoptium API）解析并缓存到 SQLite。

### 6.2 四级配方解析（cli/mod.rs `resolve_recipe_with_deps`）

```
1. 内置配方（版本 spec 为 None 或 "latest"）
   ↓ 未命中
2. 本地社区配方 ~/.oneinit/recipes/*.yaml
   ↓ 未命中（或版本不匹配）
3. 远程注册表（缓存 INDEX.json 中查找，非 versioned family）
   ↓ 未命中
4. 动态配方（versioned family + @version / 旧名重定向 python3.12→python@3.12）
```

- 旧名重定向：`old_name_redirect("python3.12")` → `("python", "3.12")`
- `install_recursive` 处理依赖（`depends`），用 `installing_stack` 防循环；使用 `BoxFuture` 实现 async 递归。

### 6.3 操作计划（Operation Plan）— dry-run 与执行共用

`core/operation.rs` 定义 14 种原子操作：`Download/Extract/CreateDir/WriteFile/AppendToFile/Delete/Exec/SetEnv/UnsetEnv/ShellCommand/PathAdd/PathRemove/CopyFile/ModifyFile`。

- `planner.rs` 负责构建计划：
  - `plan_builtin_install(recipe)` — 镜像 `recipe::install` 的语义
  - `plan_community_install(recipe, allow_exec)` — 前置执行 H-4 exec 门槛
  - `plan_uninstall(record)` — 从清单记录反向生成
- `render_plan` 渲染预览文本；`execute_plan` 依序执行，遇错即停。
- **收益**：`--dry-run` 预览与真实安装完全一致，绝不"预览一套、执行另一套"。

### 6.4 清单（Manifest）与回滚

`core/manifest.rs`：SQLite（`~/.oneinit/db/oneinit.db`，WAL 模式），表 `installed` 记录 `id/name/version/install_path/archive_url/sha256/path_entries/config_files/installed_at/original_path/env_vars_backup`。

- 安装：先备份 PATH → 执行计划 → 写入清单（记录 PATH 条目、配置文件、安装前 PATH 备份）
- 卸载：从 PATH 移除条目 → 删配置 → 删安装目录 → 删清单记录（`recipe::uninstall` / `community_recipe::uninstall`）
- `core/cache_db.rs` 使用**同一个** DB 文件，另建 `version_cache` / `checksum_cache` 表。

### 6.5 注册表（Registry）与签名

`core/registry.rs`：
- 默认注册表：`https://raw.githubusercontent.com/oneinitAI/oneinit-recipes/main`，可通过 `registry add/remove/list` 管理多个订阅（`~/.oneinit/registry.json`）
- 拉取 `{base}/INDEX.json` + 可选 `{base}/INDEX.json.sig`（Ed25519，公钥内置于 `REGISTRY_PUBLIC_KEY_HEX`）；有 `.sig` 则强校验，无则警告放行
- 多注册表合并：包名冲突时**先出现的优先**（默认注册表优先），并为每个 entry 标注 `source`
- 配方路径：`{base}/recipes/<name>/<version>.yaml`，下载后缓存到 `~/.oneinit/recipes/<name>.yaml`
- 安全 M-2：HTTP 客户端禁用重定向
- `generate_index(recipes)`：从配方列表生成 INDEX（供 publish 用）

### 6.6 团队环境同步（core/team.rs）

- `team.yaml` 结构（见 `core/sync.rs` 的 `SyncConfig`）：`team`(name/signing_key) + `envs` + `mirrors` + `env_vars` + `path` + `config_files` + `post_install`
- 安全模型：
  - 可选 Ed25519 签名：`team add` 时固定公钥（TOFU），之后每次同步强制验签
  - `config_files` / PATH 条目做路径安全检查（拒绝 `..`、必须在 home 下）
  - `post_install` 命令默认拒绝，需 `--allow-exec`
- 自动检测：`CHECK_INTERVAL_HOURS = 24`，内容哈希变化时才同步
- 应用镜像（`apply_mirrors`）：pip/npm/yarn 别名 → 真实 URL（tsinghua/aliyun/ustc/npmmirror/taobao）
- 环境变量：Windows 用 `setx`；Unix 追加到 shell profile（marker 去重）

### 6.7 安全模型总览

| 编号 | 机制 | 位置 |
|---|---|---|
| H-3 | config_files 路径逃逸检测 | `community_recipe.rs::path_escapes_install_dir` |
| H-4 | exec 门槛：含命令/安装器的配方默认拒绝，需 `--allow-exec` | `planner.rs` / `community_recipe.rs` |
| M-2 | HTTP 禁用重定向 | `registry.rs` / `team.rs` / `self_update.rs` |
| M-3 | 完整显示 SHA256（防截断/防 panic） | `community_recipe.rs` install 安全提示 |
| — | 安装前 `[SECURITY]` 确认（显示来源/哈希/命令/目标目录） | install 流程 |
| — | 注册表 INDEX.json Ed25519 验签 | `registry.rs` |
| — | 自更新 SHA256SUMS.txt 校验 | `self_update.rs` |
| — | 下载校验和（SHA256 或 SHA512，按长度自动识别） | `downloader.rs::verify_sha256` |
| — | tar 解压路径遍历防护 | `downloader.rs` |

---

## 7. 数据目录与配置文件

```
~/.oneinit/
├── envs/           # 安装的工具（install_path 与平台配方 install_path 对应）
├── db/oneinit.db   # SQLite（WAL）：清单 + 版本/校验和缓存
├── recipes/        # 本地社区配方 + 远程配方缓存
├── cache/INDEX.json# 合并后的注册表索引
├── temp/           # 下载临时文件 / 导出导入工作目录
├── registry.json   # 注册表配置（默认 URL + 订阅列表 + last_update）
├── team.json       # 团队环境配置（URL/公钥/哈希/时间戳）
└── scan_config.yaml# 自定义检测器定义（可选）
```

目录函数在 `core/mod.rs`：`data_dir/envs_dir/db_dir/temp_dir/recipes_dir/cache_dir` + `ensure_dirs()`。

---

## 8. AI 集成

### 8.1 `--json` 契约

- 每个命令输出 `{status, action, ...}`；`doctor`/`verify`/`capture`/`skill status`/`team status` 等用 `begin_document/end_document` 输出 `{action, count, items}` 聚合文档。
- `error` 输出 `{status: "error", error, suggestion}`。
- AI 工作流（详见 `.agents/skills/oneinit/SKILL.md`）：`list` → `search` → `install --dry-run` → `install` → `list` 验证；诊断用 `doctor --json` / `viz --json` / `viz --issue`。

### 8.2 Skill 安装（src/skill_mgr.rs）

- 内置 `SKILL.md`（`include_str!`）安装到：`~/.zcode/skills/oneinit/`、`~/.codex/skills/oneinit/`、`~/.claude/skills/oneinit/`、`~/.agents/skills/oneinit/`，外加子 skill `recipe-wizard/`。
- 命令：`oneinit skill install [--target zcode|codex|claude|agents|all]`。

### 8.3 内置 Skill 的核心工作流

SKILL.md 覆盖：包管理、多注册表订阅、环境捕获/迁移、预置与 sync、**项目感知安装**（`init --project`，检测 requirements.txt/pyproject.toml/package.json/Cargo.toml/go.mod）、**意图识别**（机器学习→python3.11+torch）、**故障自愈**（doctor→修复→验证）、团队环境引导、配方编写与发布、Issue/PR 引导。

---

## 9. 环境捕获与迁移

### 9.1 capture（src/core/capture/）

- `EnvDetector` trait（**同步**，无 async-trait）：`detect() -> Result<Option<RuntimeEnv>>`、`name()`、`priority()`（默认 50）
- `DetectorScheduler`：注册 7 个内置检测器（Python/Node/Git/Rust/Go/Java/Docker）+ 从 `scan_config.yaml` 加载自定义检测器，按 priority 排序后 `scan()`
- 输出 `EnvironmentSnapshot` → YAML（`oneinit.yaml` 格式）
- `detector.rs` 提供 `find_command`（where/which → PATH 遍历 → Windows 扩展名补全）、`run_command*`、`extract_version`

### 9.2 migration（src/core/migration/）

- **export**（`packer.rs`）：扫描环境 → 写 `recipe/oneinit.yaml` → 可选打包 `envs/`（含每个文件 SHA256）→ 生成 `manifest.json` → tar.gz
- **import**（`unpacker.rs`）：解压 → 读 manifest → 校验缓存文件 SHA256（失败需 `--force`）→ 恢复 recipe（`imported.yaml`）与 envs/ → 统计全局包。`--dry-run` 只预览。
- 依赖零新增：复用 flate2/tar/sha2/uuid。

---

## 10. 其他核心模块速览

- **downloader.rs**：`download`（reqwest stream + 进度条）、`compute_sha256/512`、`verify_sha256`（按长度自动选算法）、`extract`（.zip/.tar.gz，含路径遍历防护）
- **path_mgr.rs**：Windows 写 HKCU\Environment 注册表 + `PostMessageW(WM_SETTINGCHANGE)`（用非阻塞 PostMessage 避免卡死）+ `std::env::set_var`；Unix 追加到 .bashrc/.zshrc/config.fish（带 "# Added by OneInit" 标记去重）
- **config_gen.rs**：`pip_mirror_config`（清华源，Windows 用 `pip\pip.ini`）、`npm_mirror_config`、`yarn_mirror_config`
- **doctor.rs**：按类别（已安装配方/网络/PATH 环境/环境变量/系统资源/许可合规/AI 友好性）输出 `CheckResult{category,name,passed,severity,detail}`；`Severity` 为 Critical/Warning/Info；`is_healthy` = 无 Critical 失败
- **viz.rs**：纯本地数据（不发网络请求），生成 ASCII 树 / 自包含 HTML(SVG) / Issue Markdown 快照
- **self_update.rs**：GitHub API 取最新 tag → 按平台资产名 → SHA256SUMS.txt 校验 → Windows 延迟 swap 脚本 / Unix rename
- **preset.rs**：python（python3.11）、ai（python3.11）、frontend（node20）、full（python+node+go+java）
- **sync.rs**：`SyncConfig` 解析 + `envs_to_recipe_names`（python+3.11→python3.11）+ `run_post_install`

---

## 11. TUI（src/tui2/）

- `mod.rs`：入口 + 主循环（事件循环停止/重启控制）
- `backend.rs`：终端初始化/恢复 + 能力检测 + panic hook
- `event.rs`：异步事件循环（`EventStream` + mpsc + **120ms 按键去重**，处理 Windows Press+Release 成对事件）
- `screens.rs`：ratatui 渲染（标题/双面板/进度/帮助弹窗）；双面板 = Installed / Available（Tab 切换），来源标签 `[B]`/`[C]`/`[R]`
- `app.rs`：执行安装/卸载（进入执行前**必须** `drop(event_tx)` 停止事件循环，完成后重启，否则事件流与 stdin 竞争导致"按任意键无反应"）

---

## 12. 开发约定（务必遵守）

1. **源码禁用 emoji**：统一用 ASCII 标记（`[OK]`/`[ERROR]`/`[WARN]`/`[SKIP]`/`[RUN]` 等）。原因：Windows conhost 对 emoji 宽度计算错误导致布局错乱。
2. **输出必须走 `OutputFormatter`**，不要直接 `println!` 业务数据。
3. **统一错误类型**：`core::Result<T>` + `CoreError`。**禁止**引入独立 per-module error 枚举。`reqwest::Error`/`rusqlite::Error`/`zip::ZipError`/`io::Error` 已实现 `From` 自动转换。
4. **formatter.output 传 `None` 会类型推断失败**：必须用 `Some(serde_json::Value::Null)` 或具体 JSON 值。
5. **async 上下文禁止创建 `tokio::runtime::Runtime`**（会 panic），直接 `.await`。
6. **不安全代码**：Edition 2024 下 `std::env::set_var` 需要 `unsafe` 块。
7. **版本化家族列表**：`core::version::FAMILIES = ["python","node","go","java","rust"]`；新增家族需同步修改 `dynamic.rs` 的 `build` match 和 `version.rs` 的 `embedded_catalog`。
8. **动态配方 URL/校验和不得臆造**：校验和必须从官方源解析或本地计算（SKILL.md 明确"never fabricate checksums"）。
9. **开发环境**：Windows 下 Git Bash 的 GNU `link.exe` 会覆盖 MSVC 的，需先 `source dev.sh`。Rust 1.94.0 + MSVC 工具链（VS 路径见 Important.md）。
10. **工作流**：一次一任务、一次一提交；重要信息记入 `Important.md`；改动同步更新 `plan.md`。

---

## 13. 测试

- 单元测试以 `#[cfg(test)] mod tests` 内联在各模块（recipe/registry/planner/operation/community_recipe/team/version/checksum/doctor/viz/sync/cli 等均有）。
- 无集成测试框架脚本；可用 `cargo test` 运行全部。
- 注意：`manifest.rs` 的 CRUD 测试直接操作真实 `~/.oneinit/db/`（环境不支持时跳过）；`registry` 等测试多使用纯函数 + 内存数据，避免污染真实配置。

---

## 14. 常见开发任务指引

### 新增一个社区配方
1. 按 `SKILL.md`"Recipe Format"写 YAML（真实校验和！）
2. `oneinit verify my-recipe.yaml` 本地校验
3. `oneinit publish my-recipe.yaml`，把文件放入 `oneinit-recipes/recipes/<name>/<version>.yaml` 并更新 `INDEX.json`，提 PR（CI 自动校验 schema + INDEX 一致性）

### 新增一个内置配方
1. 在 `core/recipe.rs` 添加 `xxx_recipe()`（含平台 artifact 的 `url/sha256/bin_dir`）
2. 在 `resolve` / `list_recipes` 注册
3. 如需操作计划支持，确认 `planner::plan_builtin_install` 能覆盖其 install_type

### 新增一个命令
1. `main.rs` `Commands` 枚举加变体 + 分发分支
2. `cli/mod.rs` 加 `run_xxx` handler
3. 若涉及网络/自动更新判定，更新 `skip_auto_update` 与 `skip_team_check` 的 match 列表
4. 若涉及破坏性操作，加入 `is_dangerous` 判定与 `[SECURITY]` 提示
5. JSON 输出遵循 `begin_document/end_document` 或独立 `{status, action}` 结构

### 新增一个动态配方家族
1. `version.rs`：`embedded_catalog` 加条目
2. `dynamic.rs`：`build` match 加 `xxx_recipe()`
3. `checksum.rs`：`resolve` 加 `xxx_checksum()`（官方源优先）
4. `doctor.rs`：如需二进制探测，更新 `exe_for`

### 团队环境签名（推荐给团队）
见 `SKILL.md` "引导用户配置团队环境"：`scripts/sign.js --gen-key` 生成密钥对，公钥写入 `team.signing_key`，私钥设为 GitHub secret，`.github/workflows/sign.yml` 自动生成 `team.yaml.sig`。

---

## 15. 相关文档索引

| 文档 | 内容 |
|---|---|
| `README.md` / `README_CN.md` | 用户用法、安装、命令表 |
| `.agents/skills/oneinit/SKILL.md` | AI 工作流（安装/配方/迁移/团队/自愈） |
| `.agents/skills/oneinit/recipe-wizard/SKILL.md` | 无配方时的 AI 引导式配方生成流程 |
| `plan.md` | 开发计划与 Phase 状态 |
| `Important.md` | 架构笔记、代码约定、依赖清单 |
| `CONTRIBUTING.md` | 贡献指南（bug/feature/PR，CI 分级检查 S/M/L） |
| `ROADMAP.md` | 路线图 |
| `oneinit-recipes/` | 社区配方仓库（INDEX.json + recipes/） |
