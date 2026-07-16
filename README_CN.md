<div align="center">

# OneInit

**拿到一台新电脑后，第一个要装的工具。**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-green.svg)](#)
[![Tests](https://img.shields.io/badge/Tests-26%20passed-brightgreen.svg)](#)
[![AI Ready](https://img.shields.io/badge/AI-Ready%20(--json)-ff69b4.svg)](#)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust%20%E2%9D%A4-red.svg)](#)
[![Discord](https://img.shields.io/badge/PRs-Welcome-blueviolet.svg)](#)

[English](./README.md) | [中文](./README_CN.md)

</div>

---

> **OneInit** = **One** + **Init**
>
> 装完它，这台电脑就是开发者就绪的机器。一条命令，整台电脑变成开发环境。

## 为什么需要 OneInit？

| 传统方式 | OneInit |
|:--------:|:-------:|
| 下载安装包 -> 下一步 -> 下一步 -> 完成 | `oneinit install python3.11` |
| 手动配置 pip/npm 镜像源 | 自动（清华源 / 淘宝源） |
| 卸载靠 `rm -rf` 碰运气 | SQLite 清单，100% 精准回滚 |
| "旧机器上装了什么来着？" | `oneinit capture` 一键导出 |
| 新机器配环境花一整个下午 | `oneinit import backup.tar.gz` 三秒恢复 |

## 核心特性

- **完全用户态** -- 所有工具安装在 `~/.oneinit/envs/`，无需 sudo，无需管理员权限
- **自动换源** -- pip 自动走清华源，npm 自动走淘宝源，开箱即用
- **精准回滚** -- SQLite 记录每一次 PATH 修改和配置文件，卸载时 100% 还原
- **环境捕获** -- 一键扫描 Python/Node/Git/Rust/Go/Java/Docker + 自定义检测器
- **环境迁移** -- 导出完整开发环境为 tar.gz，新机器一键导入
- **社区配方** -- 写一个 YAML 文件放到 `~/.oneinit/recipes/`，一条命令安装
- **AI 原生** -- 所有命令支持 `--json` 输出，AI Agent 可直接消费

## 快速开始

```bash
# 安装 Python 3.11（含 pip + 清华源自动配置）
oneinit install python3.11

# 启动交互式 TUI 界面
oneinit tui

# 扫描当前机器环境
oneinit capture -o my-env.yaml

# 导出环境到 tar.gz
oneinit export -o backup.tar.gz --include-envs

# 在新机器上恢复
oneinit import backup.tar.gz
```

## 安装

```bash
git clone https://github.com/BG4JTS/oneinit.git
cd oneinit
cargo build --release
# 二进制文件: target/release/oneinit(.exe)
```

## 命令一览

| 命令 | 功能 |
|------|------|
| `oneinit install <包名>` | 安装工具（如 `python3.11`） |
| `oneinit uninstall <包名>` | 卸载工具（完整回滚 PATH/配置/清单） |
| `oneinit list` | 列出已安装的工具 |
| `oneinit search [关键词]` | 搜索可用配方（内置 + 社区） |
| `oneinit init --preset <名称>` | 预置套装批量安装（python/ai/frontend/full） |
| `oneinit sync` | 从 `oneinit.yaml` 批量同步环境 |
| `oneinit capture [-o 文件]` | 扫描当前环境（7 种语言检测器） |
| `oneinit export [-o 文件]` | 导出环境为 tar.gz |
| `oneinit import <文件>` | 从 tar.gz 导入环境 |
| `oneinit verify <文件>` | 验证社区配方文件 |
| `oneinit tui` | 启动交互式终端界面 |

> 所有命令均支持 `--json` 全局开关，输出结构化 JSON，AI 可直接解析。

## 环境检测器

`oneinit capture` 会检测以下环境：

| 检测器 | 收集信息 |
|--------|----------|
| Python | 版本、pip 镜像源、全局包列表 |
| Node.js | 版本、npm registry、全局包列表 |
| Git | 版本、user.name、user.email |
| Rust | rustc/cargo 版本、rustup toolchain |
| Go | 版本、GOPATH、GOROOT |
| Java | 版本、javac 版本、JAVA_HOME |
| Docker | 版本、compose 版本、容器数/镜像数 |

自定义检测器 -- 在 `~/.oneinit/scan_config.yaml` 中添加：

```yaml
custom_detectors:
  - name: flutter
    command: "flutter --version"
    version_prefix: "Flutter "
```

## 社区配方

在 `~/.oneinit/recipes/` 放一个 YAML 文件：

```yaml
name: my-tool
version: "1.0.0"
description: "一个社区配方"

platforms:
  windows:
    url: "https://example.com/tool-1.0.0.zip"
    sha256: "64位十六进制SHA256..."
    install_type: "zip_extract"
    install_path: "my-tool"
    path_add: ["{{install_dir}}"]

post_install:
  config_files:
    - path: "config.ini"
      template: "mirror = {{mirror_pip}}"
  commands:
    - "echo 安装完成"

maintainer:
  name: "作者名"
  github: "用户名"
```

然后运行：`oneinit install my-tool`

安装时会显示安全提醒（下载来源、SHA256、将执行的命令），确认后才安装。

**模板变量**：`{{install_dir}}`、`{{user_home}}`、`{{mirror_pip}}`、`{{mirror_pip_host}}`、`{{mirror_npm}}`

**支持的安装类型**：`zip_extract`、`tar_extract`、`exe_silent`、`binary_copy`、`msi_install`、`pkg_install`

## 架构

```
src/
  main.rs              CLI 入口（clap + async 命令分发）
  cli/                 命令处理器
  core/
    capture/           环境检测（7 种检测器 + 自定义）
    community_recipe   社区配方系统（YAML DTO + 加载 + 验证 + 安装）
    downloader         异步下载器 + SHA256 校验 + zip/tar.gz 解压
    manifest           SQLite 安装清单（WAL 模式，支持回滚）
    migration/         数据迁移（export tar.gz 打包 + import 解包恢复）
    path_mgr           跨平台 PATH 管理（Windows 注册表 / Unix shell 配置）
    recipe             内置配方系统（Python 3.11.9）
    sync               oneinit.yaml 批量同步
  tui2/                交互式 TUI（ratatui + crossterm 异步事件循环）
  output/              OutputFormatter（human / json 双模式）
```

## 开发

```bash
# 编译
cargo build --release

# 运行测试（26 个单元测试）
cargo test

# TUI 需要在真实终端运行（不支持管道）
oneinit tui
```

## 技术栈

| 技术 | 用途 |
|------|------|
| Rust Edition 2024 | 核心语言 |
| clap 4 | CLI 框架 |
| tokio | 异步运行时 |
| rusqlite | SQLite 清单存储 |
| ratatui + crossterm | TUI 终端界面 |
| reqwest + indicatif | 异步下载 + 进度条 |
| serde / serde_yaml / serde_json | 序列化 |

## 开源协议

GPL-3.0

---
## star
如果 OneInit 对你有帮助，欢迎 Star !
star！star！star！star！star！

---

<div align="center">

**[English](./README.md) | [中文](./README_CN.md)**

OneInit

</div>
