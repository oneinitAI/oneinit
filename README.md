# OneInit

> One command to init your dev machine -- AI-first environment initializer.

OneInit is a CLI tool that initializes a complete development environment with one command. It downloads, extracts, configures mirror sources, writes PATH, and records everything in a SQLite manifest for clean rollback.

## Quick Start

```bash
# Install Python 3.11 with pip + Tsinghua mirror auto-config
oneinit install python3.11

# Interactive TUI
oneinit tui
```

## Installation

### From source

```bash
git clone https://github.com/BG4JTS/oneinit.git
cd oneinit
cargo build --release
# Binary at target/release/oneinit(.exe)
```

## Commands

| Command | Description |
|---------|-------------|
| `oneinit install <pkg>` | Install a tool (e.g. `python3.11`) |
| `oneinit uninstall <pkg>` | Uninstall with full rollback |
| `oneinit list` | List installed tools |
| `oneinit search [keyword]` | Search available recipes (built-in + community) |
| `oneinit init --preset <name>` | Batch-install a preset suite (python/ai/frontend/full) |
| `oneinit sync` | Sync from `oneinit.yaml` in current directory |
| `oneinit capture [-o file]` | Scan current environment (7 detectors) |
| `oneinit export [-o file]` | Export environment as tar.gz |
| `oneinit import <file>` | Import environment from tar.gz |
| `oneinit verify <file>` | Validate a community recipe YAML |
| `oneinit tui` | Interactive terminal UI |

### Global flag

All commands support `--json` for AI-friendly structured output.

## Key Features

- **User-space only**: Everything installs to `~/.oneinit/envs/`. No sudo, no admin.
- **Auto mirror config**: pip uses Tsinghua, npm uses npmmirror -- automatically.
- **Clean uninstall**: SQLite manifest tracks every PATH entry and config file for 100% rollback.
- **Environment capture**: Detects Python, Node.js, Git, Rust, Go, Java, Docker + custom detectors.
- **Migration**: Export/import full dev environment as a portable tar.gz.
- **Community recipes**: Write a YAML recipe, drop it in `~/.oneinit/recipes/`, install with one command.

## Environment Detectors

`oneinit capture` detects:

| Detector | Info collected |
|----------|---------------|
| Python | version, pip mirror, global packages |
| Node.js | version, npm registry, global packages |
| Git | version, user.name, user.email |
| Rust | rustc/cargo version, rustup toolchain |
| Go | version, GOPATH, GOROOT |
| Java | version, javac version, JAVA_HOME |
| Docker | version, compose, container/image counts |

Custom detectors via `~/.oneinit/scan_config.yaml`:

```yaml
custom_detectors:
  - name: flutter
    command: "flutter --version"
    version_prefix: "Flutter "
```

## Community Recipes

Recipe YAML files in `~/.oneinit/recipes/`:

```yaml
name: my-tool
version: "1.0.0"
description: "A community recipe"

platforms:
  windows:
    url: "https://example.com/tool-1.0.0.zip"
    sha256: "64-char-hex..."
    install_type: "zip_extract"
    install_path: "my-tool"
    path_add: ["{{install_dir}}"]

post_install:
  config_files:
    - path: "config.ini"
      template: "mirror = {{mirror_pip}}"
  commands:
    - "echo done"

maintainer:
  name: "Author"
  github: "username"
```

Template variables: `{{install_dir}}`, `{{user_home}}`, `{{mirror_pip}}`, `{{mirror_pip_host}}`, `{{mirror_npm}}`

Supported `install_type`: `zip_extract`, `tar_extract`, `exe_silent`, `binary_copy`, `msi_install`, `pkg_install`

## Architecture

```
src/
  main.rs              CLI entry (clap + async dispatch)
  cli/                 Command handlers
  core/
    capture/           Environment detection (7 detectors + custom)
    community_recipe   Community recipe YAML system
    downloader         Async download + SHA256 + extract
    manifest           SQLite install manifest
    migration/         Export/import (tar.gz packer + unpacker)
    path_mgr           Cross-platform PATH management
    recipe             Built-in recipe system
    sync               oneinit.yaml batch sync
  tui2/                Interactive TUI (ratatui + crossterm)
  output/              OutputFormatter (human + JSON)
```

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# The TUI requires a real terminal (not piped)
oneinit tui
```

## License

GPL-3.0
