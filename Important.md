# OneInit 重要笔记

## 项目结构
```
src/
├── main.rs              # CLI 入口（clap 定义 + async 命令分发）
├── cli/
│   └── mod.rs            # 各子命令处理器（调用配方/预置/同步系统）
├── core/
│   ├── mod.rs            # CoreError 统一错误类型 + 目录函数
│   ├── capture/          # 环境捕获模块（mod.rs + detector + python + node + git + rust + go + java + docker）
│   ├── community_recipe.rs # 社区配方系统（YAML DTO + 加载 + 验证 + 安装）
│   ├── downloader.rs     # 异步下载器 + SHA256 校验 + 归档解压
│   ├── manifest.rs       # SQLite 安装清单系统
│   ├── migration/        # 数据迁移模块（mod + manifest + packer + unpacker）
│   ├── path_mgr.rs       # 跨平台 PATH 管理
│   ├── config_gen.rs     # 配置生成器（自动换源）
│   ├── recipe.rs         # 内置配方系统（Recipe 结构 + 安装/卸载执行器）
│   ├── registry.rs       # 社区配方注册表（INDEX.json + 远程 fetch + 缓存）
│   ├── preset.rs         # 预置套装（Preset 结构 + 内置套装定义）
│   └── sync.rs           # 同步系统（SyncConfig 结构 + oneinit.yaml 解析）
├── tui2/
│   ├── mod.rs            # TUI 入口 + 主循环（事件循环停止/重启控制）
│   ├── backend.rs        # 终端初始化/恢复 + 能力检测 + panic hook
│   ├── event.rs          # 异步事件循环（EventStream + mpsc + 120ms 去重）
│   ├── screens.rs        # ratatui 渲染（标题/双面板/进度/帮助弹窗）
│   ├── state.rs          # AppState + Screen 路由 + Pane 焦点
│   └── app.rs            # 操作执行（退出TUI→install/uninstall→任意键返回）
└── output/
    └── mod.rs            # OutputFormatter（human / json 双模式 + error）
```

## 数据目录
```
~/.oneinit/
├── envs/          # 安装的工具（如 python3.7/, node18/）
├── db/
│   └── oneinit.db # SQLite 清单数据库（WAL 模式）
└── temp/          # 下载临时文件
```

## 开发环境

### 编译
- **问题**：Git Bash 的 GNU `link.exe` 覆盖了 MSVC 的 `link.exe`
- **解决**：每次开发前执行 `source dev.sh`，将 MSVC 工具链放到 PATH 前面
- **VS 路径**：`D:/Program Files/Microsoft Visual Studio/18/Enterprise/`
- **Rust 版本**：1.94.0 (x86_64-pc-windows-msvc)

### Cargo 配置
- `.cargo/config.toml` 中配置了 MSVC LIBPATH

## 错误处理
- **CoreError 枚举**（`core::CoreError`）：统一错误类型
  - Download / Checksum / Extract / Database / PathOp / ConfigGen / Io / Other
- **Result 别名**：`core::Result<T>` = `std::result::Result<T, CoreError>`
- **自动转换**：`reqwest::Error` / `rusqlite::Error` / `zip::ZipError` / `io::Error`
- **输出**：`OutputFormatter::error()` 统一格式化

## 命令设计

### 子命令
| 命令 | 用途 | 状态 |
|------|------|------|
| `oneinit init` | 一键初始化开发环境 | ✅ 预置套装批量安装 |
| `oneinit install <pkg>` | 安装指定工具 | ✅ 配方系统集成 |
| `oneinit uninstall <pkg>` | 卸载指定工具 | ✅ 完整回滚 |
| `oneinit list` | 列出已安装工具 | ✅ 已接入 SQLite |
| `oneinit search [kw]` | 搜索可用工具 | ✅ 已接入配方注册表 |
| `oneinit sync` | 从 oneinit.yaml 同步 | ✅ YAML 解析 + 批量安装 + 后置命令 |
| `oneinit capture [-o file]` | 捕获当前环境 | ✅ Python/Node/Git/Rust/Go/Java/Docker 检测 + YAML |
| `oneinit export [-o file]` | 导出环境快照 | ✅ tar.gz 打包（环境 + 可选 envs + manifest.json） |
| `oneinit import <file>` | 导入环境快照 | ✅ 解压 + 校验 + 恢复配方/envs/包列表 |
| `oneinit update` | 更新远程配方索引 | ✅ 拉取 INDEX.json 到缓存（类似 apt update） |
| `oneinit publish <file>` | 发布配方到远程仓库 | ✅ 验证 + 安全提醒 + git PR 步骤 |
| `oneinit verify <file>` | 验证社区配方文件 | ✅ YAML 语法 + 字段 + SHA256 + install_type |
| `oneinit tui` | 启动交互式 TUI 界面 | ✅ ratatui 异步事件循环 + 双面板 |

### 全局开关
- `--json`：所有命令支持 JSON 输出，AI 可直接消费

## 核心模块 API

### downloader.rs
```rust
pub async fn download(url, dest) -> Result<DownloadResult>
pub fn compute_sha256(path) -> Result<String>
pub fn verify_sha256(path, expected) -> Result<bool>
pub fn extract(archive, dest) -> Result<Vec<PathBuf>>   // .zip / .tar.gz
```

### manifest.rs
```rust
Manifest::open() -> Result<Manifest>
manifest.add(record) -> Result<String>          // 返回 ID
manifest.get(name) -> Result<Option<Record>>
manifest.list() -> Result<Vec<Record>>
manifest.remove(name) -> Result<Option<Record>> // 返回记录用于回滚
```

### path_mgr.rs
```rust
path_mgr::add(directory) -> Result<()>
path_mgr::remove(directory) -> Result<()>
path_mgr::backup() -> Result<String>
path_mgr::restore(backup) -> Result<()>
```

### config_gen.rs
```rust
config_gen::apply_configs(install_dir, configs) -> Result<Vec<PathBuf>>
config_gen::remove_configs(install_dir, configs) -> Result<()>
config_gen::pip_mirror_config() -> AppConfig   // 清华源
config_gen::npm_mirror_config() -> AppConfig   // 淘宝镜像
config_gen::yarn_mirror_config() -> AppConfig  // 淘宝镜像
```

### recipe.rs（配方系统）
```rust
// 配方结构
pub struct Recipe {
    name, version, display_name, download_url, sha256, bin_dir,
    env_vars: Vec<(String, String)>,
    configs: Vec<AppConfig>,
    post_install: Option<PostInstall>,
}
pub enum PostInstallStep {
    DownloadAndRun { url, args },   // 下载脚本并执行
    ModifyFile { rel_path, action }, // 修改文件
}
pub enum ModifyAction {
    UncommentLine { pattern },    // 取消注释
    AppendLine { content },        // 追加行
    ReplaceContent { content },    // 替换内容
}

// 配方注册表
pub fn resolve(name: &str) -> Option<Recipe>
pub fn list_recipes() -> Vec<Recipe>

// 安装/卸载执行器
pub async fn install(recipe, formatter) -> Result<()>
pub async fn uninstall(package, formatter) -> Result<()>
```

### 已实现配方
| 包名 | 版本 | 来源 | 说明 |
|------|------|------|------|
| `python3.11` | 3.11.9 | python.org embeddable | Windows 嵌入式包 + get-pip 引导 + 清华源 |

### community_recipe.rs（社区配方系统）
严格按 `社区配方.md` 实现。配方 YAML 格式见该文档 1.1 节。
```rust
// DTO（serde::Deserialize）
pub struct CommunityRecipe { name, version, description, platforms, post_install?, depends?, tags?, maintainer? }
pub struct Platforms { windows?, linux?, darwin? }  // 各为 PlatformConfig
pub struct PlatformConfig { url, sha256, install_type, install_args?, install_path, path_add }

// 加载与查找
pub fn load_all() -> Vec<CommunityRecipe>              // 扫描 ~/.oneinit/recipes/*.yaml
pub fn resolve(name) -> Option<CommunityRecipe>
pub fn verify(yaml_path) -> Result<VerifyResult>       // 验证配方文件

// 安装（含安全提醒）
pub async fn install(recipe, formatter) -> Result<()>  // [SECURITY] 确认后下载安装
pub async fn uninstall(name, formatter) -> Result<()>

// 模板渲染
pub fn render_template(template, install_dir) -> String  // {{install_dir}} {{mirror_pip}} 等
```

**安全提醒**：安装前醒目显示下载来源、SHA256、写入目录、将执行的命令，等用户输入 y 确认。
**install_type 支持**：`zip_extract`、`tar_extract`、`exe_silent`、`binary_copy`（其他报错）
**集成方式**：`run_install` 先查内置 `recipe::resolve()`，未命中查社区配方；`run_search` 合并显示

### preset.rs（预置套装）
```rust
pub struct Preset { name, display_name, description, packages: Vec<String> }
pub fn resolve(name: &str) -> Option<Preset>
pub fn list_presets() -> Vec<Preset>
```

内置套装：`python`（Python 3.11）、`ai`（AI 开发）、`frontend`（前端，暂空）、`full`（全栈）

### sync.rs（同步系统）
```rust
// oneinit.yaml 结构
pub struct SyncConfig {
    pub envs: BTreeMap<String, Value>,        // python: 3.11
    pub mirrors: Option<BTreeMap<String, String>>, // pip: tsinghua
    pub post_install: Option<Vec<String>>,    // shell 命令列表
}
pub fn load_config(yaml_path) -> Result<SyncConfig>  // 解析 YAML
pub fn envs_to_recipe_names(config) -> Vec<String>  // envs → recipe 名映射
pub fn run_post_install(commands, formatter) -> Result<()>  // 执行后置命令
```

oneinit.yaml 格式：
```yaml
envs:
  python: 3.11
mirrors:
  pip: tsinghua
post_install:
  - pip install -r requirements.txt
```

## 代码约定
- **Edition**：2024
- **禁止 emoji**：源码（.rs）中禁止使用 emoji 字符，统一用 ASCII 标记代替（`[OK]` `[ERROR]` `[WAIT]` `[PKG]` `[LIST]` `[SEARCH]` `[SUCCESS]` `[DEL]` `[SKIP]` `[START]` `[DONE]` `[RUN]` `[CONF]` `[WARN]`）。原因：Windows 传统 conhost 对部分 emoji 宽度计算错误，导致终端布局错乱。
- **输出格式**：通过 `OutputFormatter` 统一管理，永远不要直接 `println!` 业务数据
- **错误处理**：`core::Result<T>` + `CoreError` 枚举
- **unsafe**：`std::env::set_var` 在 edition 2024 需要 unsafe 块
- **formatter.output 类型推断**：传 `None` 时类型推断失败，必须用 `Some(serde_json::Value::Null)` 或具体 JSON 值
- **已处于 async 上下文**：不要在 async 函数中创建 `tokio::runtime::Runtime`（会 panic），直接 `.await` 即可
- **TUI 退出执行模式**：执行安装/卸载前必须 `drop(event_tx)` 停止事件循环，否则 `EventStream` 会和 `stdin` 读取竞争导致"按任意键无反应"。执行后重启事件循环。
- **Windows 按键去重**：crossterm 在 Windows 发送 Press+Release 成对事件，事件循环层用 120ms 去重 + Release 过滤
- **TUI 模块路径为 `tui2/`**（非 `tui/`），因旧 `tui/` 目录残留权限问题无法删除
- **社区配方格式**：严格按 `社区配方.md` 实现，声明式 YAML，支持模板变量（`{{install_dir}}` 等），详见 `社区配方.md`
- **Phase 4 架构适配（关键！）**：设计文档 `数据的采集与迁移.md` 中的代码示例不能直接使用，必须适配：
  - 统一用 `CoreError`（新增 `Capture(String)` / `Migration(String)` 变体），不引入独立 per-module error 枚举
  - 模块路径用 `src/core/capture/` 和 `src/core/migration/`（非独立 crate）
  - 依赖版本必须对齐：`winreg = "0.55"` / `dirs = "6"`（文档写的旧版本）
  - Edition 2024 原生 async trait 可替代 `async-trait` crate，需验证
  - MVP 先只做 gzip（flate2 已有），zstd 后续扩展
  - `tempfile` 需测试 Windows 行为，可能用现有 `temp_dir()`

## Phase 4 新增依赖（待添加）
| crate | 版本 | 用途 | 阶段 |
|-------|------|------|------|
| async-trait | 0.1 | 检测器 trait（或用 Edition 2024 原生） | 4A |
| walkdir | 2 | 递归遍历目录 | 4B |
| tempfile | 3 | 安全临时目录 | 4B |
| gethostname | 0.4 | 导出时记录主机名 | 4B |
| regex | 1 | 版本号解析 | 4A |

## 依赖清单
| crate | 版本 | 用途 |
|-------|------|------|
| clap | 4.x | CLI 框架（derive 模式） |
| tokio | 1.x (full) | 异步运行时 |
| serde | 1.x | 序列化基础 |
| serde_json | 1.x | JSON 输出 |
| serde_yaml | 0.9.x | YAML 配置解析 |
| indicatif | 0.18.x | 下载进度条 |
| thiserror | 2.x | 错误类型定义 |
| reqwest | 0.12.x (stream) | HTTP 异步下载 |
| futures-util | 0.3.x | Stream 支持 |
| sha2 | 0.10.x | SHA256 校验 |
| flate2 | 1.x | gzip 解压 |
| tar | 0.4.x | tar 归档解压 |
| zip | 2.x | zip 解压 |
| rusqlite | 0.32.x (bundled) | SQLite 清单存储 |
| uuid | 1.x (v4) | manifest ID 生成 |
| winreg | 0.55.x | Windows 注册表（cfg(windows)） |
| winapi | 0.3.x | Windows API（cfg(windows)） |
| dirs | 6.x | 跨平台 home 目录 |
| chrono | 0.4.x | 时间戳 |
| ratatui | 0.29.x (no default-features) | TUI 终端界面渲染 |
| crossterm | 0.28.x (event-stream) | 终端后端 + 异步事件 |
| tokio-stream | 0.1.x | EventStream 异步迭代 |
