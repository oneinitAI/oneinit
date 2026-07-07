# OneInit 重要笔记

## 项目结构
```
src/
├── main.rs              # CLI 入口（clap 定义 + async 命令分发）
├── cli/
│   └── mod.rs            # 各子命令处理器（调用核心引擎）
├── core/
│   ├── mod.rs            # CoreError 统一错误类型 + 目录函数
│   ├── downloader.rs     # 异步下载器 + SHA256 校验 + 归档解压
│   ├── manifest.rs       # SQLite 安装清单系统
│   ├── path_mgr.rs       # 跨平台 PATH 管理
│   └── config_gen.rs     # 配置生成器（自动换源）
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
| `oneinit init` | 一键初始化开发环境 | Phase 2 |
| `oneinit install <pkg>` | 安装指定工具 | 引擎就绪，等配方 |
| `oneinit uninstall <pkg>` | 卸载指定工具 | 清单已集成 |
| `oneinit list` | 列出已安装工具 | ✅ 已接入 SQLite |
| `oneinit search [kw]` | 搜索可用工具 | Phase 2 |
| `oneinit sync` | 从 oneinit.yaml 同步 | Phase 2 |

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

## 代码约定
- **Edition**：2024
- **输出格式**：通过 `OutputFormatter` 统一管理，永远不要直接 `println!` 业务数据
- **错误处理**：`core::Result<T>` + `CoreError` 枚举
- **unsafe**：`std::env::set_var` 在 edition 2024 需要 unsafe 块

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
