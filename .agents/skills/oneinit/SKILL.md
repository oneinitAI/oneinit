---
name: oneinit
description: >
  OneInit developer environment initializer — install, configure, and
  migrate dev tools with one command. Use when the user wants to: set up
  a new dev machine, install Python/Node/Rust/Go/Java/Git/Docker, configure
  pip/npm mirror sources (Tsinghua/npmmirror), capture current environment
  snapshot, export/import dev setup as tar.gz, search available packages,
  verify community recipes, or batch-sync from oneinit.yaml. Trigger on
  phrases like "init dev environment", "install python", "set up new
  machine", "mirror source", "capture environment", "export dev setup",
  "migrate tools", "oneinit", "configure pip mirror", "bootstrap dev".
---

# OneInit — AI-Driven Dev Environment Management

OneInit is a CLI tool that initializes a complete development environment with one command. It downloads, extracts, configures mirrors, writes PATH, and records everything in a SQLite manifest for clean rollback.

**Key principle**: Always pass `--json` to get structured, parseable output. Human-readable mode is for interactive use only.

## Quick Reference

| Intent | Command |
|--------|---------|
| Install a tool | `oneinit install <package> --json` |
| Uninstall a tool | `oneinit uninstall <package> --json` |
| List installed tools | `oneinit list --json` |
| Search available tools | `oneinit search [keyword] --json` |
| Capture current environment | `oneinit capture [-o file] --json` |
| Export environment to tar.gz | `oneinit export [-o file] [--include-envs] --json` |
| Import environment from tar.gz | `oneinit import <file> [--dry-run] [--force] --json` |
| Init with preset suite | `oneinit init --preset <name>` |
| Sync from oneinit.yaml | `oneinit sync --json` |
| Verify a community recipe | `oneinit verify <file> --json` |
| Launch interactive TUI | `oneinit tui` |

## Commands

### Package Management

#### Install a tool

```bash
# Install Python 3.11 (includes pip + Tsinghua mirror auto-config)
oneinit install python3.11 --json

# Check if already installed before installing
oneinit list --json
```

Install flow: download -> SHA256 verify -> extract -> post-install (get-pip etc.) -> apply mirror configs -> add to PATH -> record in manifest.

**Security**: Community recipes display a `[SECURITY]` confirmation prompt before installing. The user must type `y` to proceed. For automated/AI workflows, prefer built-in recipes (like `python3.11`) which do not require confirmation.

#### Uninstall a tool

```bash
oneinit uninstall python3.11 --json
```

Performs full rollback: removes PATH entries, deletes config files, removes install directory, deletes manifest record.

#### List and Search

```bash
# List all installed tools
oneinit list --json
# Returns: { "status": "success", "installed": [...], "count": N }

# Search available recipes (built-in + community)
oneinit search python --json
# Returns results with "source": "builtin" or "community"
```

### Environment Capture

Scans the current machine for installed dev tools and generates a reproducible YAML snapshot.

```bash
# Capture to default file
oneinit capture --json

# Capture to custom path
oneinit capture -o my-env.yaml --json
```

Detected environments (7 built-in detectors):
- **Python**: version, pip mirror, global packages (`pip list --format=freeze`)
- **Node.js**: version, npm registry, global npm packages
- **Git**: version, user.name, user.email
- **Rust**: rustc/cargo version, rustup toolchain
- **Go**: version, GOPATH, GOROOT
- **Java**: version (from stderr), javac version, JAVA_HOME
- **Docker**: version, compose version, container/image counts

Custom detectors: Users can add entries to `~/.oneinit/scan_config.yaml`:

```yaml
custom_detectors:
  - name: flutter
    command: "flutter --version"
    version_prefix: "Flutter "
```

### Migration (Export / Import)

#### Export

```bash
# Lightweight export (environment metadata only, ~3 KB)
oneinit export -o backup.tar.gz --json

# Full export (includes installed tool binaries from ~/.oneinit/envs/)
oneinit export -o backup.tar.gz --include-envs --json
```

The tar.gz contains:
- `recipe/oneinit.yaml` — captured environment snapshot
- `manifest.json` — migration manifest with checksums
- `cache/` — tool binaries (only with `--include-envs`)

#### Import

```bash
# Preview what would be restored (no changes made)
oneinit import backup.tar.gz --dry-run --json

# Actually import (restores recipe + package lists)
oneinit import backup.tar.gz --json

# Force overwrite existing files
oneinit import backup.tar.gz --force --json
```

Import flow: extract tar.gz -> verify SHA256 checksums -> restore recipe to `~/.oneinit/recipes/imported.yaml` -> optionally restore envs cache -> record package lists for manual or `oneinit sync` installation.

### Presets and Sync

#### Init with Preset

```bash
# List available presets
oneinit init

# Install Python development suite
oneinit init --preset python

# Available presets: python, ai, frontend, full
```

#### Sync from oneinit.yaml

```bash
oneinit sync --json
```

Reads `oneinit.yaml` from the current directory, installs all listed tools, applies mirror config, and runs post-install commands.

`oneinit.yaml` format:

```yaml
envs:
  python: 3.11

mirrors:
  pip: tsinghua

post_install:
  - pip install -r requirements.txt
```

### Community Recipes

#### Verify a Recipe File

```bash
oneinit verify my-recipe.yaml --json
```

Checks: YAML syntax, required fields (name/version/description), platform coverage, SHA256 length (64 chars), install_type validity, maintainer warning.

#### Community Recipe Format

Recipe YAML files go in `~/.oneinit/recipes/`. Once placed there, they are automatically discoverable by `oneinit search` and installable by `oneinit install <name>`.

```yaml
name: my-tool
version: "1.0.0"
description: "A community recipe"

platforms:
  windows:
    url: "https://example.com/tool-1.0.0.zip"
    sha256: "64-char-hex-string-here..."
    install_type: "zip_extract"
    install_path: "my-tool"
    path_add: ["{{install_dir}}"]

post_install:
  config_files:
    - path: "config.ini"
      template: |
        [global]
        mirror = {{mirror_pip}}
  commands:
    - "echo setup complete"

tags:
  - "utility"

maintainer:
  name: "Author"
  github: "username"
```

**Template variables** (auto-replaced in path_add, config_files, commands):
- `{{install_dir}}` — absolute install path
- `{{user_home}}` — user home directory
- `{{mirror_pip}}` — `https://pypi.tuna.tsinghua.edu.cn/simple`
- `{{mirror_pip_host}}` — `pypi.tuna.tsinghua.edu.cn`
- `{{mirror_npm}}` — `https://registry.npmmirror.com`

**Supported install_type values**: `zip_extract`, `tar_extract`, `exe_silent`, `binary_copy`

## AI Best Practices

1. **Always use `--json`** for programmatic output parsing. The JSON includes `status`, `action`, and all relevant data fields.

2. **Check before installing**: Run `oneinit list --json` first to avoid duplicate installs. Already-installed packages return `already_installed: true`.

3. **Search before assuming**: Run `oneinit search <keyword> --json` to verify a recipe exists before attempting `oneinit install`.

4. **Capture before migrating**: On the source machine, run `oneinit capture --json` and review the output. Then `oneinit export --include-envs --json` for a complete backup.

5. **Dry-run imports**: Always run `oneinit import <file> --dry-run --json` first to preview what will be restored.

6. **Community recipe safety**: Community recipe installs require interactive `y` confirmation (displays download source, SHA256, commands to execute). For AI-driven workflows, prefer built-in recipes or pre-verified community recipes.

7. **Path refresh**: After `oneinit install`, the tool is added to PATH via Windows registry or Unix shell config. The user may need to open a new terminal for PATH changes to take effect.
