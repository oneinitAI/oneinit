---
name: oneinit-recipe-wizard
description: Guide the AI agent through the recipe-wizard flow — when a tool has no OneInit recipe, the AI generates a step-by-step tutorial, helps build a community recipe YAML, installs it with oneinit, and offers the contribution flow. Use when a tool has no existing OneInit recipe, when the user asks for a manual install guide, or when the user wants to generate/save/contribute a recipe.
---

# OneInit Recipe Wizard (分 Skill) — AI 驱动的配方向导

This sub-skill replaces the old interactive CLI wizard. The **AI is the wizard**:
it uses its own knowledge to write tutorials and recipes, and oneinit only
executes them. Trigger it whenever:

- `oneinit search <tool>` / `oneinit install <tool>` reports no recipe found
- The user asks for a tool that has no OneInit recipe yet
- The user wants a manual install guide, or wants to generate / contribute a recipe

## Flow 1 — No recipe found: give the user two choices

When an install fails with "Not found", do NOT leave the user stuck. Present
two options and let the user pick:

1. **教程 (Tutorial)** — the AI writes a step-by-step manual install guide for
   the user to follow by hand (see below).
2. **生成配方 (Generate)** — the AI builds a community recipe YAML, oneinit
   installs it, and afterwards offer the contribution flow.

### Choice A — AI-generated tutorial

Write a clear, platform-aware tutorial. Never just dump a URL — give the user
an ordered checklist:

1. **前置检查**：先让用户确认目标平台（windows/linux/darwin）与架构（x64/arm64），
   并运行 `oneinit list --json` 确认该工具尚未安装。
2. **下载**：给出官方下载页/直链，注明当前最新稳定版本号。
3. **安装**：给出该平台的安装方式（zip 解压后加 PATH / 安装器 / 包管理器），
   具体到目录和命令。Windows 提示 PATH 修改或使用新终端。
4. **验证**：给出验证命令（如 `<tool> --version`）判断安装成功。
5. **配置（可选）**：镜像源、环境变量、配置文件等后续配置要点。

### Choice B — AI-generated recipe + oneinit install

1. **Gather facts (never guess URLs or checksums)** — ask the user for, or
   research yourself:
   - Official download URL **per platform** (windows/linux/darwin)
   - **Real checksum**: fetch from the official SHA256SUMS/sidecar file, or
     download + compute locally. A wrong checksum makes the recipe fail.
   - `install_type` (see list below) and the correct `install_path`/`path_add`
     (account for top-level dirs in archives — they are NOT stripped)
   - Optional mirror config for the package manager (pip→Tsinghua, npm→npmmirror,
     cargo→rsproxy.cn, maven→Aliyun, nuget→cnblogs)
2. **Write the recipe YAML** to `~/.oneinit/recipes/<name>.yaml`:

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

   - **Template variables**: `{{install_dir}}`, `{{user_home}}`, `{{mirror_pip}}`,
     `{{mirror_pip_host}}`, `{{mirror_npm}}`
   - **install_type**: `zip_extract`, `tar_extract`, `exe_silent`, `binary_copy`,
     `msi_install`, `pkg_install`
   - **Platform keys**: `windows`, `linux`, `darwin` (at least one required)
3. **Validate locally**:
   ```bash
   oneinit verify ~/.oneinit/recipes/<name>.yaml --json
   ```
   Fix anything it reports before installing.
4. **Install with oneinit**:
   ```bash
   oneinit install <name> --json
   ```
   - If the recipe has `post_install.commands`, oneinit gates execution behind
     `--allow-exec` — pass it explicitly only after showing the user what will run.
   - Community recipes show a `[SECURITY]` confirmation prompt; use `-y` to
     auto-confirm when the user already approved.
5. **Verify**: `oneinit list --json` shows the tool as installed; run
   `<tool> --version` to confirm.

## Flow 2 — Contribution (after the environment is configured)

After a successful setup, always ask: **"是否愿意贡献这个配方？"** If yes,
offer two paths and let the user choose:

### Path A — 网页上传（零门槛）

Upload the recipe to the OneInit website (it creates a PR on your behalf):

```bash
curl -X POST https://oneinit.bg4jts.cn/api/v1/recipes \
  -H "Content-Type: application/yaml" \
  --data-binary @~/.oneinit/recipes/<name>.yaml
```

- Server validates: `name` format, at least one platform with a valid http(s)
  `url`, `install_type` validity, `version` presence.
- Response: `{ "ok": true, "branch": ..., "pull_request_url": ..., "pull_request_number": ... }`.
  Give the user the PR link to track review.

### Path B — Git PR（有门槛，适合有 GitHub 经验者）

Guide the user through: fork `oneinitAI/oneinit-recipes` → add
`recipes/<name>/<version>.yaml` → update `INDEX.json` (alphabetical) →
open a PR. CI validates schema + INDEX consistency automatically.
Also mention `oneinit publish <file>` for the repo workflow and
`python scripts/validate.py` (in the recipes repo) for local validation.

## Rules

- **Never fabricate** a download URL or checksum — always use real values from
  official sources, or compute the checksum from the downloaded artifact.
- An empty `sha256` means "skip verification" — warn the user about the risk.
- A generated recipe only covers the platforms the user needs; community
  recipes ideally cover windows/linux/darwin — encourage multi-platform.
- Prefer `--json` for all oneinit invocations in this flow so you can parse
  results programmatically.
- `post_install.commands` are arbitrary code execution — always disclose what
  they do and only pass `--allow-exec` with explicit user consent.
- The tutorial is a fallback for users who prefer manual control; if the user
  wants automation, steer them to Choice B (generate + install).
