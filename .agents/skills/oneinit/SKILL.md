---
name: oneinit
description: >
  OneInit developer environment initializer — install, configure, and
  migrate dev tools with one command. Use when the user wants to: set up
  a new dev machine, install Python/Node/Rust/Go/Java/MySQL/.NET, configure
  pip/npm/cargo/maven/nuget mirror sources, subscribe to custom recipe
  registries, generate/verify/publish community recipes, capture current
  environment snapshot, export/import dev setup as tar.gz, search available
  packages, batch-sync from oneinit.yaml, or file issues/PRs against the
  oneinit repositories. Trigger on phrases like "init dev environment",
  "install python", "set up new machine", "mirror source", "capture
  environment", "export dev setup", "migrate tools", "recipe registry",
  "oneinit", "recipe generator", "write a recipe", "submit issue", "PR".
---

# OneInit — AI-Driven Dev Environment Management

OneInit is a CLI tool that initializes a complete development environment with one command. It downloads, extracts, configures mirrors, writes PATH, and records everything in a SQLite manifest for clean rollback.

**Key principle**: Always pass `--json` to get structured, parseable output. Human-readable mode is for interactive use only.

**Auto-update**: Every `oneinit` invocation silently refreshes the recipe index when the cache is missing or older than 24h — no manual `update` needed for normal use.

## Quick Reference

| Intent | Command |
|--------|---------|
| Install a tool | `oneinit install <package> --json` |
| Uninstall a tool | `oneinit uninstall <package> --json` |
| List installed tools | `oneinit list --json` |
| Search available tools | `oneinit search [keyword] --json` |
| Update recipe index (all registries) | `oneinit update --json` |
| Add a registry subscription | `oneinit registry add <url>` |
| List registry subscriptions | `oneinit registry list` |
| Remove a registry subscription | `oneinit registry remove <url>` |
| Capture current environment | `oneinit capture [-o file] --json` |
| Export environment to tar.gz | `oneinit export [-o file] [--include-envs] --json` |
| Import environment from tar.gz | `oneinit import <file> [--dry-run] [--force] --json` |
| Init with preset suite | `oneinit init --preset <name>` |
| Sync from oneinit.yaml | `oneinit sync --json` |
| Verify a community recipe | `oneinit verify <file> --json` |
| Health check | `oneinit doctor --json` |
| Export installed tools (like pip freeze) | `oneinit freeze -o file.yaml` |
| Generate shell completion | `oneinit completions bash` |
| Install AI Skill to agents | `oneinit skill install` |
| Launch interactive TUI | `oneinit tui` |

## Built-in Recipes (no confirmation, always available)

| Name | Version | Description |
|------|---------|-------------|
| `python3.11` | 3.11.9 | Python + pip + Tsinghua mirror (Windows) |
| `node20` | 20.18.1 | Node.js 20 LTS + npm npmmirror (win/linux/mac) |
| `go` | 1.23.4 | Go toolchain (win/linux/mac) |
| `java17` | 17.0.20+8 | Temurin JDK 17 LTS (win/linux/mac) |

Community registry (`oneinit-recipes`) also provides: `rust` (rustup + rsproxy.cn mirror), `dotnet8` (.NET SDK 8 / C# + NuGet cnblogs mirror), `mysql8` (MySQL 8.0), and more.

## Commands

### Package Management

#### Install a tool

```bash
# Install Python 3.11 (includes pip + Tsinghua mirror auto-config)
oneinit install python3.11 --json

# Install a community/remote recipe (auto-fetched from registry)
oneinit install dotnet8 --json

# Check if already installed before installing
oneinit list --json

# Preview operations without executing (install/uninstall/init/sync/team sync)
oneinit install python3.11 --dry-run --json
oneinit uninstall python3.11 --dry-run --json

# Skip interactive confirmations / debug output (global flags)
oneinit -y install node20
oneinit -v install node20
```

#### Aliases

`i`→install · `u`/`rm`→uninstall · `ls`→list · `up`→sync · `check`→doctor · `upgrade`→self-update

#### Self-update

```bash
oneinit self-update --json    # or: oneinit upgrade
```
Downloads the latest release asset, verifies against SHA256SUMS.txt, then
replaces the running binary (Windows uses a delayed swap script).

Install flow: download -> checksum verify (SHA256 or SHA512) -> extract -> post-install -> apply mirror configs -> add to PATH -> record in manifest.

**Security**: Community recipes display a `[SECURITY]` confirmation prompt before installing. The user must type `y` to proceed. For automated/AI workflows, prefer built-in recipes (like `python3.11`).

#### Uninstall a tool

```bash
oneinit uninstall python3.11 --json
```

Performs full rollback: removes PATH entries, deletes config files, removes install directory, deletes manifest record.

#### List and Search

```bash
# List all installed tools
oneinit list --json

# Search available recipes (builtin + community + remote)
oneinit search java --json
# Returns results with "source": "builtin" | "community" | "remote"
```

### Registry Subscriptions (multi-registry)

OneInit can pull recipes from **multiple** registries. The default is `oneinitAI/oneinit-recipes`; custom registries are merged in (default wins on name conflicts).

```bash
# Add a custom registry (must provide INDEX.json at {url}/INDEX.json)
oneinit registry add "https://raw.githubusercontent.com/yourname/recipes/main"

# List subscriptions
oneinit registry list

# Remove a subscription
oneinit registry remove "https://raw.githubusercontent.com/yourname/recipes/main"

# Force-refresh all registries
oneinit update
```

A registry repo must contain: `INDEX.json` (package index) + `recipes/<name>/<version>.yaml` (recipe files).

### Environment Capture

```bash
# Capture to default file
oneinit capture --json

# Capture to custom path
oneinit capture -o my-env.yaml --json
```

Detected environments (7 built-in detectors): Python, Node.js, Git, Rust, Go, Java, Docker. Custom detectors via `~/.oneinit/scan_config.yaml`.

### Migration (Export / Import)

```bash
oneinit export -o backup.tar.gz --json                          # metadata only
oneinit export -o backup.tar.gz --include-envs --json           # + binaries
oneinit import backup.tar.gz --dry-run --json                   # preview
oneinit import backup.tar.gz --json                             # restore
```

### Presets and Sync

```bash
oneinit init                          # list presets
oneinit init --preset python          # python suite
oneinit init --preset frontend        # node20
oneinit init --preset full            # python + node + go + java
oneinit sync --json                   # batch-install from oneinit.yaml
```

### Team Environment Sync (团队环境同步)

团队共享开发环境：fork `oneinitAI/oneinit-team-env` 模板，编辑 `team.yaml`，成员一次配置后
**每次运行 oneinit 自动检测同步**（工具 / 镜像 / 环境变量 / PATH / 配置文件）。

```bash
# 配置团队环境（拉取 team.yaml + 验签 + 固定公钥，随后立即同步）
oneinit team add https://raw.githubusercontent.com/<org>/<repo>/main --json
oneinit team add https://github.com/<org>/<repo> --branch main --json   # github.com 形式也可

# 手动同步 / 强制同步
oneinit team sync --json
oneinit team sync --force --json      # 忽略 24h 间隔与缓存哈希

# 状态 / 移除
oneinit team status --json
oneinit team remove
```

- team.yaml 结构：`team`(name/signing_key) + `envs` + `mirrors` + `env_vars` + `path` + `config_files` + `post_install`，见 `docs/团队环境.md`
- 签名（可选）：`team add` 固定公钥（TOFU），每次同步强制验签，不匹配拒绝
- 安全：缺失工具逐个 `y` 确认；执行命令类配方默认拒绝（`--allow-exec`）；配置文件写入前预览确认
- 自动检测：每次运行检查，24h 内未变则零网络开销；失败静默不阻塞主命令

#### 引导用户配置团队环境（Guide: 帮用户搭建团队环境时按此流程）

1. **创建/编辑环境仓库**：让用户 fork `github.com/oneinitAI/oneinit-team-env` 模板，
   或直接帮用户改 `team.yaml`（工具 `envs`、镜像 `mirrors`、环境变量 `env_vars`、
   PATH `path`、配置文件 `config_files`）。模板变量：`{{user_home}}` `{{mirror_pip}}` `{{mirror_npm}}`。
2. **（可选，推荐）签名**：
   - 本地：`cd <repo> && node scripts/sign.js --gen-key` → 得到 `TEAM_SIGN_KEY`(私钥 seed) 与 `signing_key`(公钥)
   - 把公钥写入 `team.yaml` 的 `team.signing_key`；私钥设为 GitHub secret：
     `gh secret set TEAM_SIGN_KEY --repo <org>/<repo> --body <seed-hex>`（私钥不要提交进代码库）
   - push 后 `.github/workflows/sign.yml` 自动生成 `team.yaml.sig`（未配置 secret 则跳过签名）
3. **成员接入**：`oneinit team add https://raw.githubusercontent.com/<org>/<repo>/main`
   （或 `oneinit team add https://github.com/<org>/<repo>`，自动转 raw；`--branch` 指定分支；
   换过密钥/覆盖配置用 `--force` 重新固定公钥）
4. **验证**：`oneinit team status`（看 URL/团队名/签名状态）、`oneinit team sync --force`
   （立即同步，缺失工具逐个确认）
5. **常见问题**：
   - 成员报"签名不匹配"→ 团队换过密钥，让成员 `team add <url> --force` 重新固定
   - `post_install` 没执行 → 默认拒绝远程命令，需 `oneinit team sync --allow-exec`
   - 新工具没同步 → 每 24h 自动检测一次，或 `team sync --force` 立即同步

### Environment Visualization (oneinit viz)

```bash
# ASCII 环境树（工具/版本/激活状态/全局包/缓存/磁盘占用）
oneinit viz --json

# 跳过全局包扫描（快速模式，不运行 pip/npm 检测）
oneinit viz --no-scan --json

# HTML(SVG) 报告（自包含，可 --open 打开浏览器）
oneinit viz --html -o report.html --open

# GitHub Issue 环境快照（Markdown，直接粘贴到 Issue，节省沟通成本）
oneinit viz --issue -o env-snapshot.md
```

- 数据全部来自本机磁盘（Manifest/envs/cache/recipes），不发起网络请求（自动跳过索引更新）
- `(active)` = 该工具的 bin 目录在当前 PATH 中；`⚠ dir missing` = 安装目录已丢失
- 全局包：复用 capture 检测器（python/node），`--no-scan` 可跳过

### TUI

```bash
oneinit tui
```

- Two panes: **Installed** / **Available** (Tab to switch)
- Available pane shows source tags: `[B]` builtin (green), `[C]` community (yellow), `[R]` remote (cyan)
- Enter to install/uninstall, `c` to capture, `r` to refresh, `?` for help
- Starts with an automatic index refresh when stale

## Community Recipes

### Verify a Recipe File

```bash
oneinit verify my-recipe.yaml --json
```

Checks: YAML syntax, required fields, platform coverage, checksum format (64-char SHA256 **or** 128-char SHA512), install_type validity, maintainer warning.

### Recipe Format

Recipe YAML files in `~/.oneinit/recipes/` are auto-discovered by `search` and `install`.

```yaml
name: my-tool
version: "1.0.0"
description: "A community recipe"

platforms:
  windows:
    url: "https://example.com/tool-1.0.0.zip"
    sha256: "64-char-hex-or-128-char-sha512..."
    install_type: "zip_extract"
    install_path: "my-tool"
    path_add: ["{{install_dir}}"]

post_install:
  config_files:
    - path: "{{user_home}}/.config/my-tool.conf"
      template: "mirror = {{mirror_pip}}"
  commands:
    - "echo setup complete"

tags:
  - "utility"

maintainer:
  name: "Author"
  github: "username"
```

**Template variables**: `{{install_dir}}`, `{{user_home}}`, `{{mirror_pip}}`, `{{mirror_pip_host}}`, `{{mirror_npm}}`

**Supported install_type**: `zip_extract`, `tar_extract`, `exe_silent`, `binary_copy`, `msi_install`, `pkg_install`

**Platform keys**: `windows`, `linux`, `darwin` (at least one required). Note: archives extract WITHOUT stripping the top-level directory, so `path_add` must point at the real binary dir (e.g. `{{install_dir}}/node-v20.18.1-linux-x64`).

## Generating Recipes (recipe authoring guide)

When the user asks to create a recipe, follow this workflow:

### 1. Gather facts (never guess checksums)

- Official download URL per platform (windows/linux/darwin)
- **Real checksum**: fetch from the official SHA256SUMS/sidecar file, or download + compute locally. A wrong checksum makes the recipe fail.
- `install_type` (see supported list) and correct `install_path`/`path_add` (account for top-level dirs in archives)
- Mirror source config for the package manager (pip→Tsinghua, npm→npmmirror, cargo→rsproxy.cn, maven→Aliyun, nuget→cnblogs)

### 2. Write the recipe YAML

Follow the format above. Use `{{mirror_*}}` template variables for mirror configs. Write config files to `{{user_home}}/...` so the tool reads them globally.

### 3. Validate locally

```bash
oneinit verify my-recipe.yaml
python scripts/validate.py    # in the oneinit-recipes repo (schema + INDEX consistency)
```

### 4. Publish

```bash
oneinit publish my-recipe.yaml
```

Then submit a PR to [oneinitAI/oneinit-recipes](https://github.com/oneinitAI/oneinit-recipes) following the contribution steps it prints. CI automatically validates the PR.

## Contributing — Issues & PRs

OneInit has **two** repos; file issues in the right one:

| Repo | Purpose | Issue templates |
|------|---------|-----------------|
| [oneinitAI/oneinit](https://github.com/oneinitAI/oneinit) | Core CLI, TUI, bugs, features | Bug / Feature / Recipe request |
| [oneinitAI/oneinit-recipes](https://github.com/oneinitAI/oneinit-recipes) | Recipe registry, recipe bugs | Recipe request / Recipe bug |

### Guiding users to file issues

When a user reports a bug or asks for a feature/recipe, guide them to:

1. **Bug**: `oneinit --version` + OS + reproduction steps + full error output (use `--json` for structured errors). File at oneinit → "🐛 Bug Report" (auto-labels `bug`).
2. **Feature**: describe the pain point, desired behavior, alternatives. oneinit → "✨ Feature Request" (`enhancement`).
3. **Recipe**: package name, version, official download URL, target platforms, mirror suggestion. oneinit-recipes → "📦 Recipe Request" (`recipe`).

Provide the user a ready-to-paste issue body so they don't have to fill forms from scratch.

### Guiding users to submit PRs

- **Code changes** (main repo): branch from `main` → make changes → open PR. CI runs tiered checks: `S` (≤10 files: fmt+clippy+test), `M` (11-30: +release build), `L` (>30: +cross-platform check). Labels: `docs` (link check only), `ci` (workflow YAML check), `breaking` (forces L).
- **Recipe additions** (recipes repo): add `recipes/<name>/<version>.yaml` + update `INDEX.json` (alphabetical) → PR. CI validates schema + INDEX consistency automatically.
- **Branch protection**: `main` requires CI to pass + 1 review. Direct pushes are rejected — always use a PR.
- After the PR, tell the user to check the CI status and wait for review/merge.

## AI Best Practices

1. **Always use `--json`** for programmatic output parsing. The JSON includes `status`, `action`, and all relevant data fields.

2. **Check before installing**: Run `oneinit list --json` first to avoid duplicate installs. Already-installed packages return `already_installed: true`.

3. **Search before assuming**: Run `oneinit search <keyword> --json` to verify a recipe exists before attempting `oneinit install`. Sources: `builtin` / `community` / `remote`.

4. **No manual update needed**: the index auto-refreshes on use. Run `oneinit update` explicitly only when you need the latest index immediately (e.g. after a new recipe is published).

5. **Multi-registry**: if a package isn't found, suggest `oneinit registry add <url>` if the user has a private/custom registry.

6. **Capture before migrating**: On the source machine, run `oneinit capture --json` and review the output. Then `oneinit export --include-envs --json` for a complete backup.

7. **Dry-run imports**: Always run `oneinit import <file> --dry-run --json` first to preview what will be restored.

8. **Community recipe safety**: Community recipe installs require interactive `y` confirmation (displays download source, checksum, commands to execute). For AI-driven workflows, prefer built-in recipes or pre-verified community recipes.

9. **Path refresh**: After `oneinit install`, the tool is added to PATH via Windows registry or Unix shell config. The user may need to open a new terminal for PATH changes to take effect.

10. **Never fabricate checksums** in recipes: always fetch the real value from official sources or compute it from the downloaded artifact.
