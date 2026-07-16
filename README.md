<div align="center">

# OneInit

**One command to init your dev machine.**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-green.svg)](#)
[![Tests](https://img.shields.io/badge/Tests-26%20passed-brightgreen.svg)](#)
[![AI Ready](https://img.shields.io/badge/AI-Ready%20(--json)-ff69b4.svg)](#)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust%20%E2%9D%A4-red.svg)](#)

[English](./README.md) | [中文](./README_CN.md)

</div>

---

OneInit is a CLI tool that initializes a complete development environment with **one command**. Download, extract, configure mirrors, write PATH, record in SQLite -- all automatic, all user-space, zero sudo.

## Why OneInit?

| Traditional | OneInit |
|-------------|---------|
| Download installer -> Next -> Next -> Finish | `oneinit install python3.11` |
| Manually configure pip/npm mirrors | Auto (Tsinghua / npmmirror) |
| `rm -rf` and hope for clean uninstall | SQLite manifest, 100% rollback |
| "What did I install on my old machine?" | `oneinit capture` -> `oneinit export` |
| New machine setup takes an afternoon | `oneinit import backup.tar.gz` |

## Quick Start

```bash
# Install Python 3.11 with pip + Tsinghua mirror auto-config
oneinit install python3.11

# Interactive TUI
oneinit tui

# Scan what's installed on this machine
oneinit capture -o my-env.yaml
```

## Installation

```bash
git clone https://github.com/BG4JTS/oneinit.git
cd oneinit
cargo build --release
# Binary: target/release/oneinit(.exe)
```

## Commands

| Command | Description |
|---------|-------------|
| `oneinit install <pkg>` | Install a tool (e.g. `python3.11`) |
| `oneinit uninstall <pkg>` | Uninstall with full rollback |
| `oneinit list` | List installed tools |
| `oneinit search [keyword]` | Search available recipes |
| `oneinit init --preset <name>` | Batch-install a suite (python/ai/frontend/full) |
| `oneinit sync` | Batch sync from `oneinit.yaml` |
| `oneinit capture [-o file]` | Scan current environment (7 detectors) |
| `oneinit export [-o file]` | Export environment as tar.gz |
| `oneinit import <file>` | Import environment from tar.gz |
| `oneinit verify <file>` | Validate a community recipe YAML |
| `oneinit tui` | Interactive terminal UI |

> All commands support `--json` for AI-friendly structured output.

## Environment Detectors

`oneinit capture` detects Python, Node.js, Git, Rust, Go, Java, Docker + custom detectors.

## Community Recipes

Drop a YAML file in `~/.oneinit/recipes/`:

```yaml
name: my-tool
version: "1.0.0"
description: "A tool"
platforms:
  windows:
    url: "https://example.com/tool.zip"
    sha256: "64-char-hex..."
    install_type: "zip_extract"
    install_path: "my-tool"
    path_add: ["{{install_dir}}"]
maintainer:
  name: "You"
  github: "yourname"
```

Then: `oneinit install my-tool`

## License

GPL-3.0
