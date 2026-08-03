# Team Environment Sync

> Share one dev environment across a team: tools, mirrors, env vars, PATH,
> config files. The team maintains a GitHub repo; members fork/configure once,
> and **oneinit auto-detects and syncs on every run**.

## Architecture

```
User machine                           Team repo (oneinit-team-env, forkable)
┌─────────────────┐   ── fetch ──>   ┌──────────────────────────┐
│ ~/.oneinit/     │   <─ .sig (opt) ─ │ team.yaml (env def)      │
│  team.json      │                   │ team.yaml.sig (Ed25519)  │
│  url / key/hash │                   │ scripts/sign.js          │
└─────────────────┘                   │ .github/workflows/sign.yml│
   │ every run                        └──────────────────────────┘
   ▼
 maybe_team_sync (main.rs startup hook)
 1. not configured → skip; <24h & hash unchanged → skip (zero cost)
 2. changed/first run → fetch + verify signature → confirm each missing tool
    → apply mirrors / env_vars / PATH / config_files
```

## Quick Start

```bash
# 1. Team forks the https://github.com/oneinitAI/oneinit-team-env template
# 2. Edit team.yaml (format below), push to main
# 3. Every member runs once:
oneinit team add https://raw.githubusercontent.com/<org>/<repo>/main

# 4. From then on oneinit auto-detects and syncs; manual force sync:
oneinit team sync --force
```

## team.yaml Format

```yaml
team:
  name: "WebTeam"               # team name (shown by status)
  description: "Web team shared dev environment"
  version: "1"                  # env definition version
  signing_key: "d8ee1d1c..."    # optional: Ed25519 public key hex (with team.yaml.sig)

envs:                           # tools (3-tier: builtin / local community / remote registry)
  node: "20"                    # → recipe name node20
  python: "3.11"                # → recipe name python3.11

mirrors:                        # mirrors (written to user-level config, idempotent)
  pip: "tsinghua"               # alias: tsinghua/aliyun/ustc, or full URL
  npm: "npmmirror"              # alias: npmmirror/taobao, or full URL
  # yarn: "npmmirror"

env_vars:                       # env vars (Unix: shell profile; Windows: setx)
  NODE_ENV: "development"
  TEAM_NPM_REGISTRY: "https://registry.npmmirror.com"

path:                           # PATH additions (template variables supported)
  - "{{user_home}}/myteam/bin"

config_files:                   # config file templates (preview + y/N confirm)
  - path: "{{user_home}}/.npmrc"
    template: |
      registry={{mirror_npm}}

post_install:                   # post-install commands (safety: denied by default, needs --allow-exec)
  - "echo 'Welcome to myteam!'"
```

Template variables (in `config_files` / `path`):
`{{user_home}}` `{{install_dir}}` `{{mirror_pip}}` `{{mirror_pip_host}}` `{{mirror_npm}}`

## Commands

| Command | Description |
|---------|-------------|
| `oneinit team add <url> [--branch main] [--force] [--allow-exec]` | Configure team env: fetch team.yaml + verify signature + pin key (TOFU), then sync immediately. `--force` overwrites old config / re-pins the key |
| `oneinit team sync [--force] [--allow-exec] [--dry-run]` | Sync now. `--force` ignores the 24h interval and cached hash; `--dry-run` only previews |
| `oneinit team status` | Show URL / team name / signature state / last check & sync time |
| `oneinit team remove` | Remove the team env config |

Supported URLs: `https://github.com/<org>/<repo>`, `https://github.com/<org>/<repo>.git`,
`https://github.com/<org>/<repo>/tree/<branch>`, `https://raw.githubusercontent.com/<org>/<repo>/<branch>`.

## Auto-Detection

- **Frequency**: checked on every oneinit run; skipped when <24h since last
  check (local config only, zero network).
- **Change detection**: fetch `team.yaml` (+`.sig`), compare SHA256 against the
  local cache; unchanged → only refresh the timestamp.
- **Sync**: when changed or `--force`, each missing tool installs with the
  security banner + `y` confirm, then mirrors / env vars / PATH / config files /
  post_install are applied.
- **Failures never block**: a failed check/fetch only emits `[WARN]`.
- **Partial failure**: if not all tools installed, the synced hash is
  **not recorded** — retry later (or `--force`).

## Signature Verification (optional, recommended)

- The team declares `signing_key` (Ed25519 public key hex) in `team.yaml` and
  provides `team.yaml.sig` at the repo root.
- On `team add`, oneinit verifies the signature and **pins the public key**
  (Trust-On-First-Use).
- Every sync afterwards verifies: mismatched key / bad signature / declared key
  without `.sig` → **sync refused**.
- Unsigned repos work normally (with a `[WARN]` hint).
- Signing flow (already built into the template repo):
  ```bash
  node scripts/sign.js --gen-key        # generates TEAM_SIGN_KEY (seed) + signing_key (public)
  gh secret set TEAM_SIGN_KEY --repo <org>/<repo> --body <seed>   # private key goes only into the GitHub secret
  # put the public key into team.yaml's team.signing_key; the Action signs on push
  ```
- After the team rotates keys, members re-run `oneinit team add <url> --force`.

## Security Model

| Risk | Mitigation |
|------|------------|
| Malicious / tampered team.yaml | optional Ed25519 verification (pinned key), mismatch refused |
| Tool installs run commands | reuse `--allow-exec` default-deny; security banner + `y` per tool |
| Arbitrary post_install commands | skipped by default; run only with `--allow-exec` (auto-sync never runs them) |
| Config files written anywhere | preview + `y/N`; path must be an absolute path under home without `..` |
| PATH entries escaping | `..` entries refused; path_mgr is idempotent |
| Sync failure blocking the main command | failures are silent `[WARN]`, never blocking |

## vs `oneinit sync`

| | `oneinit sync` | `oneinit team sync` |
|---|---|---|
| Source | local `oneinit.yaml` | remote team repo `team.yaml` |
| Auto-detection | none | every run (24h interval) |
| Signature | none | optional Ed25519 verification |
| Scope | envs + mirrors (logged only) + post_install | envs + mirrors (applied) + env_vars + PATH + config_files + post_install |
