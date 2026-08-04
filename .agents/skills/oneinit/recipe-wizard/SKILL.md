---
name: oneinit-recipe-wizard
description: Guide the AI agent through OneInit's recipe wizard — when a tool has no recipe, offer tutorial/generate choices and the contribution flow. Use when the user needs a tool with no existing OneInit recipe.
---

# OneInit Recipe Wizard (分 Skill)

This sub-skill extends the main OneInit skill with the **recipe wizard** flow.
Trigger it whenever:

- `oneinit install <tool>` reports "Not found: recipe"
- The user asks for a tool that has no OneInit recipe yet
- The user wants to generate, save, or contribute a recipe

## Flow 1 — No recipe found (install time)

When an install fails with "Not found", OneInit itself offers the wizard
interactively. If the user prefers to delegate to you (the agent), run the
wizard on their behalf:

1. Ask the user to choose:
   - **Tutorial** → run `oneinit recipe tutorial <tool>` and walk them through
     the manual steps (or read the output and help them).
   - **Generate** → run `oneinit recipe wizard <tool>` and collect from the
     user: the download URL (required), sha256 (optional), install type
     (inferred from URL if skipped), PATH dir (optional), description.
2. After generation, `oneinit` saves the recipe to `~/.oneinit/recipes/<tool>.yaml`
   and installs it. If the recipe needs `--allow-exec`, pass it explicitly.

## Flow 2 — Contribution

After the environment is configured, offer to contribute the recipe:

1. Ask: "是否愿意贡献这个配方?" — if yes:
2. Two options:
   - **Upload**: `oneinit recipe contribute <file>` → option 1 (oneinit.bg4jts.cn).
     Note: the upload backend is NOT live yet — the interface is reserved.
     The recipe stays at `~/.oneinit/recipes/<tool>.yaml`.
   - **Git**: print the fork → branch → copy → PR steps (option 2) and help the
     user execute them against `oneinitAI/oneinit-recipes`.

## Rules

- Never fabricate a download URL — ask the user for the real one.
- An empty sha256 means "skip verification" — warn the user about the risk.
- The generated recipe only covers the current platform; mention that
  community recipes should ideally cover windows/linux/darwin.
- Respect `--json` / `-y` modes: the wizard is interactive, so skip prompts
  when the CLI runs non-interactively.
