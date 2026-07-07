# OneInit 重要笔记

## 项目结构
```
src/
├── main.rs          # CLI 入口（clap 定义 + 命令分发）
├── cli/
│   └── mod.rs        # 各子命令的处理器
├── core/
│   └── mod.rs        # 核心引擎（下载、清单、PATH、配置）
└── output/
    └── mod.rs        # OutputFormatter（human / json 双模式）
```

## 开发环境

### 编译
- **问题**：Git Bash 的 GNU `link.exe` 覆盖了 MSVC 的 `link.exe`
- **解决**：每次开发前执行 `source dev.sh`，将 MSVC 工具链放到 PATH 前面
- **VS 路径**：`D:/Program Files/Microsoft Visual Studio/18/Enterprise/`
- **Rust 版本**：1.94.0 (x86_64-pc-windows-msvc)

### Cargo 配置
- `.cargo/config.toml` 中配置了 MSVC LIBPATH

## 命令设计

### 子命令
| 命令 | 用途 | Phase |
|------|------|-------|
| `oneinit init` | 一键初始化开发环境 | Phase 2 |
| `oneinit install <pkg>` | 安装指定工具 | Phase 1 |
| `oneinit uninstall <pkg>` | 卸载指定工具 | Phase 1 |
| `oneinit list` | 列出已安装工具 | Phase 1 |
| `oneinit search [kw]` | 搜索可用工具 | Phase 2 |
| `oneinit sync` | 从 oneinit.yaml 同步 | Phase 2 |

### 全局开关
- `--json`：所有命令支持 JSON 输出，AI 可直接消费

## 代码约定
- **Edition**：2024
- **输出格式**：通过 `OutputFormatter` 统一管理，永远不要直接 `println!` 业务数据
- **错误处理**：待确定（Phase 1 任务 2 中统一设计）

## 依赖清单
| crate | 版本 | 用途 |
|-------|------|------|
| clap | 4.x | CLI 框架（derive 模式） |
| tokio | 1.x (full) | 异步运行时 |
| serde | 1.x | 序列化基础 |
| serde_json | 1.x | JSON 输出 |
| serde_yaml | 0.9.x | YAML 配置解析 |
| indicatif | 0.18.x | 下载进度条 |
