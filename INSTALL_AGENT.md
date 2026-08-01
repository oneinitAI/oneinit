# OneInit AI Installation Guide

> This file is designed for AI agents. Copy the prompt below and paste it to your AI assistant to install OneInit and set up your development environment automatically.

---

## Prompt for AI (English)

```
Install OneInit on this machine and set up my development environment.

Option A — One-line install (recommended, no prerequisites):
  curl -fsSL https://raw.githubusercontent.com/oneinitAI/oneinit/main/install.sh | sh

Option B — Install via npm:
  npm install -g oneinit

Option C — Build from source (requires Rust):
  git clone https://github.com/oneinitAI/oneinit.git
  cd oneinit && cargo build --release
  # Then add target/release to PATH

After installation:

1. Verify:
   oneinit --version
   oneinit doctor

2. Update recipe registry:
   oneinit update

3. Install tools (use --json for parseable output):
   oneinit install python3.11 --json
   oneinit install node20 --json

4. Verify tools:
   python --version
   npm --version

5. Configure AI Skill (for ZCode/Claude/Codex):
   The skill is bundled at .agents/skills/oneinit/SKILL.md in the repo.
   For ZCode: copy to ~/.zcode/skills/oneinit/
   For Codex: copy to ~/.codex/skills/oneinit/
   Generic:   copy to ~/.agents/skills/oneinit/

Report each step's result.
```

## Prompt for AI (Chinese / 中文)

```
在这台机器上安装 OneInit 并配置开发环境。

方式 A — 一键安装（推荐，无需任何前置依赖）：
  curl -fsSL https://raw.githubusercontent.com/oneinitAI/oneinit/main/install.sh | sh

方式 B — 通过 npm 安装：
  npm install -g oneinit

方式 C — 从源码编译（需要 Rust）：
  git clone https://github.com/oneinitAI/oneinit.git
  cd oneinit && cargo build --release
  # 然后将 target/release 添加到 PATH

安装完成后：

1. 验证安装：
   oneinit --version
   oneinit doctor

2. 更新配方仓库索引：
   oneinit update

3. 安装工具（使用 --json 获取可解析输出）：
   oneinit install python3.11 --json
   oneinit install node20 --json

4. 验证工具是否正常：
   python --version
   npm --version

5. 配置 AI Skill（适用于 ZCode/Claude/Codex）：
   Skill 文件位于仓库 .agents/skills/oneinit/SKILL.md。
   ZCode: 复制到 ~/.zcode/skills/oneinit/
   Codex: 复制到 ~/.codex/skills/oneinit/
   通用:  复制到 ~/.agents/skills/oneinit/

报告每一步的结果。
```

---

## Version Management via npm

```bash
npm install -g oneinit          # install latest
npm install -g oneinit@0.1.0    # install specific version
npm update -g oneinit           # upgrade to latest
npm uninstall -g oneinit        # uninstall
npm list -g oneinit             # check installed version
```

## Available Recipes

| Package | Version | Install Command |
|---------|---------|-----------------|
| Python | 3.11.9 | `oneinit install python3.11` |
| Node.js | 20.18.1 | `oneinit install node20` |

> Run `oneinit update && oneinit search` to see all available recipes.

## Version Syntax (oneinit packages)

```bash
oneinit install python@latest      # latest version
oneinit install node@20.18.1       # specific version
```

## Environment Migration

```bash
# On old machine:
oneinit freeze -o oneinit.yaml     # export installed tools
oneinit export -o backup.tar.gz --include-envs  # full backup

# On new machine:
oneinit import backup.tar.gz       # restore everything
oneinit sync                       # install from oneinit.yaml
```
