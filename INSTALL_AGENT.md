# OneInit AI Installation Guide

> This file is designed for AI agents. Copy the prompt below and paste it to your AI assistant to install OneInit and set up your development environment automatically.

---

## Prompt for AI (English)

```
Install OneInit on this machine and set up my development environment.

Steps:
1. Clone and build OneInit:
   git clone https://github.com/BG4JTS/oneinit.git
   cd oneinit
   cargo build --release

2. Add to PATH:
   - Windows: Add F:\oneinit\target\release to PATH
   - Linux/macOS: export PATH="$PATH:~/oneinit/target/release"

3. Verify installation:
   oneinit --version
   oneinit doctor

4. Update recipe registry:
   oneinit update

5. Install tools (use --json for parseable output):
   oneinit install python3.11 --json
   oneinit install node20 --json

6. Verify tools are working:
   python --version
   npm --version

7. Configure AI Skill (for ZCode/Claude/Codex):
   The skill file is at .agents/skills/oneinit/SKILL.md.
   Copy it to your agent's skills directory:
   - ZCode: ~/.zcode/skills/oneinit/
   - Codex: ~/.codex/skills/oneinit/
   - Generic: ~/.agents/skills/oneinit/

Report each step's result.
```

## Prompt for AI (Chinese / 中文)

```
在这台机器上安装 OneInit 并配置开发环境。

步骤：
1. 克隆并编译 OneInit：
   git clone https://github.com/BG4JTS/oneinit.git
   cd oneinit
   cargo build --release

2. 添加到 PATH：
   - Windows: 将 F:\oneinit\target\release 添加到 PATH
   - Linux/macOS: export PATH="$PATH:~/oneinit/target/release"

3. 验证安装：
   oneinit --version
   oneinit doctor

4. 更新配方仓库索引：
   oneinit update

5. 安装工具（使用 --json 获取可解析输出）：
   oneinit install python3.11 --json
   oneinit install node20 --json

6. 验证工具是否正常：
   python --version
   npm --version

7. 配置 AI Skill（适用于 ZCode/Claude/Codex）：
   Skill 文件位于 .agents/skills/oneinit/SKILL.md。
   复制到你的 AI 助手的 skills 目录：
   - ZCode: ~/.zcode/skills/oneinit/
   - Codex: ~/.codex/skills/oneinit/
   - 通用: ~/.agents/skills/oneinit/

报告每一步的结果。
```

---

## Available Recipes

| Package | Version | Install Command |
|---------|---------|-----------------|
| Python | 3.11.9 | `oneinit install python3.11` |
| Node.js | 20.18.1 | `oneinit install node20` |

> Run `oneinit update && oneinit search` to see all available recipes.

## Version Syntax

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
