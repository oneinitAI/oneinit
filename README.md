<div align="center">

<img src="https://picui.ogmua.cn/s1/2026/08/01/6a6d8fa53e5ca.webp" alt="OneInit Logo" width="320" />

# OneInit

**One command to init your dev machine.**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-green.svg)](#)
[![AI Ready](https://img.shields.io/badge/AI-Ready%20(--json)-ff69b4.svg)](#)
[![PRs-Welcome](https://img.shields.io/badge/PRs-Welcome-blueviolet.svg)](#)

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

### Option 1: One-line install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/oneinitAI/oneinit/main/install.sh | sh
```

No prerequisites. The script detects your OS/architecture, downloads the pre-built binary, installs to `~/.oneinit/bin/`, and configures PATH automatically.

### Option 2: Install via npm

```bash
npm install -g oneinit
```

Version management:
```bash
npm install -g oneinit          # install latest
npm install -g oneinit@0.1.0    # specific version
npm update -g oneinit           # upgrade
npm uninstall -g oneinit        # remove
```

### Option 3: Build from source

```bash
git clone https://github.com/oneinitAI/oneinit.git
cd oneinit
cargo build --release
# Binary: target/release/oneinit(.exe)
```

### Option 4: Ask AI to install for you

Copy this prompt to your AI assistant (ChatGPT, Claude, ZCode, etc.):

**English prompt:**

```
Install OneInit on this machine and set up my dev environment.
Follow the guide at:
https://raw.githubusercontent.com/oneinitAI/oneinit/main/INSTALL_AGENT.md
```

**中文提示词：**

```
在这台机器上安装 OneInit 并配置开发环境。
按照以下指南操作：
https://raw.githubusercontent.com/oneinitAI/oneinit/main/INSTALL_AGENT.md
```

The guide includes: npm or build instructions, PATH setup, tool installation (Python, Node.js), AI Skill configuration, and environment migration.

## Commands

| Command | Description |
|---------|-------------|
| `oneinit install <pkg>` | Install a tool (e.g. `python3.11`) |
| `oneinit uninstall <pkg>` | Uninstall with full rollback |
| `oneinit list` | List installed tools |
| `oneinit search [keyword]` | Search available recipes |
| `oneinit init --preset <name>` | Batch-install a suite (python/ai/frontend/full) |
| `oneinit sync` | Batch sync from `oneinit.yaml` |
| `oneinit install <pkg> --dry-run` | Preview operations without executing |
| `oneinit self-update` | Update OneInit itself (SHA256SUMS-verified) |
| `oneinit team add <url>` | Configure team env repo (verify signature, pin key) |
| `oneinit team sync` | Sync team env now (auto-checked on every run) |
| `oneinit team status` / `remove` | Show / remove team env config |
| `oneinit viz` | Visualize environment as an ASCII tree |
| `oneinit viz --html` | Generate HTML(SVG) environment report |
| `oneinit viz --issue` | Generate paste-ready GitHub Issue snapshot |
| `oneinit capture [-o file]` | Scan current environment (7 detectors) |
| `oneinit export [-o file]` | Export environment as tar.gz |
| `oneinit import <file>` | Import environment from tar.gz |
| `oneinit verify <file>` | Validate a community recipe YAML |
| `oneinit tui` | Interactive terminal UI |

> All commands support `--json` for AI-friendly structured output.
> Global flags: `-y/--yes` skips confirmations, `-v/--debug` enables debug
> output. Aliases: `i`→install, `u`/`rm`→uninstall, `ls`→list, `up`→sync,
> `check`→doctor, `upgrade`→self-update.

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

## Team Environment Sync

Share one dev environment across your team. Fork the
[oneinit-team-env](https://github.com/oneinitAI/oneinit-team-env) template,
edit `team.yaml` (tools, mirrors, env vars, PATH, config files), push — and
every member stays in sync automatically.

```bash
# one-time setup (per member)
oneinit team add https://raw.githubusercontent.com/<org>/<repo>/main

# auto: every `oneinit` run detects changes (24h interval) and syncs
# manual: force sync now
oneinit team sync --force
```

- **Signature (optional, recommended):** `team.yaml` can be Ed25519-signed;
  oneinit pins the key on `team add` and verifies on every sync. Tampered
  content is rejected.
- **Safety:** missing tools install with the usual security banner + `y`
  confirmation; recipes that run commands/installers require
  `--allow-exec` (default deny); config files are previewed before writing.
- See [docs/team-env.md](docs/team-env.md) for the full spec. 中文版见
  [docs/团队环境.md](docs/团队环境.md).

## Contributing & Roadmap

- **[CONTRIBUTING.md](CONTRIBUTING.md)** — report bugs, propose features,
  submit PRs, code style. Newcomer-friendly, FAQ included. 中文版见
  [CONTRIBUTING_CN.md](CONTRIBUTING_CN.md).
- **[ROADMAP.md](ROADMAP.md)** — project direction & where to contribute value.
  中文版见 [ROADMAP_CN.md](ROADMAP_CN.md).
- **Good First Issues** — beginner-friendly tasks with clear acceptance
  criteria: https://github.com/oneinitAI/oneinit/issues?q=label%3A%22good+first+issue%22
- Communication happens on GitHub Issues & PR comments.

## Terms of Service

By using OneInit you agree to the following:

- **Automation guidance only.** OneInit automates downloading and installing
  software from the URLs declared in recipes. It does **not** host, store,
  redistribute, or endorse any software copies.
- **Your responsibility.** You are solely responsible for the software you
  install and for complying with its license, copyright, and local laws.
  OneInit is not a party to your relationship with the software publishers.
- **Respect licenses.** Before installing, review the `license` /
  `license_url` shown in the `[SECURITY]` confirmation prompt. If a tool's
  license does not permit the intended use, do not install it.

## Security & Disclaimer

OneInit downloads files from the internet, modifies your PATH, writes config files, and may execute install scripts. By using OneInit you acknowledge:

- **Community recipes are not audited.** Always review the `[SECURITY]` prompt before installing (it shows the download URL, SHA256, and commands that will run).
- **Use `oneinit verify`** to validate recipe files before installing.
- **Use `oneinit doctor`** to check for environment issues.
- OneInit is provided **"as is" without warranty** under GPL-3.0. The authors are not liable for any damage.

## License

GPL-3.0

## Support

OneInit is a solo open-source project. If it saves you time, consider supporting:

[![Sponsor](https://img.shields.io/badge/Sponsor-❤-ea4aaa?logo=githubsponsors&logoColor=white)](https://github.com/sponsors/BG4JTS)
[![Open Collective](https://img.shields.io/badge/Open%20Collective-3385FF?logo=opencollective&logoColor=white)](https://opencollective.com/bg4jts)

**Every star ⭐ helps more developers discover OneInit.** Star the repo to show your support!

## 爱发电 / Afdian

国内朋友也可以直接在爱发电支持我：

[![爱发电](https://img.shields.io/badge/爱发电-支持我-946CE6?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0iIzc2NkNDMCI+PHBhdGggZD0iTTE4LjkgMTIuNWwtNi4zIDUuMmMtLjMuMi0uNy4yLTEgMGwtNi4zLTUuMmMtLjQtLjMtLjQtLjgtLjEtMS4xbDIuMS0yLjJjLjMtLjMuOC0uMyAxLjEgMGw0LjIgMy41IDQuMi0zLjVjLjMtLjMuOC0uMyAxLjEgMGwyLjEgMi4yYy4zLjMuMy44LS4xIDEuMXoiLz48L3N2Zz4=)](https://ifdian.net/a/BG4JTS)

→ **https://ifdian.net/a/BG4JTS**

## Credits / 致谢

I'm a developer — this code was written by me, with heavy assistance from
AI coding tools: **DeepSeek, GLM (智谱), ChatGPT, and others**. Large parts of
the codebase were written, reviewed, and debugged with their help.
Thanks to all the models that made this project possible!

> 我是程序员——本项目的代码由我编写，并大量借助 **DeepSeek、GLM（智谱）、
> ChatGPT** 等 AI 编程助手完成。感谢这些模型帮助编写、审查和调试了大量代码。

---
<div align="center">

**[English](./README.md) | [中文](./README_CN.md)**

OneInit

</div>