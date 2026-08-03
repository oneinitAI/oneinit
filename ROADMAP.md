# OneInit Roadmap

> Project goals and direction. Here you'll find what's been shipped and what's
> next — and where you can contribute value.

## ✅ Released

**v0.1.0-beta series** (current: `v0.1.0-beta.2`)

- ✅ One-command init: builtin recipes (Python/Node/Go/Java) + auto mirror
      config (pip Tsinghua / npm npmmirror)
- ✅ Community recipe system: YAML recipes + registry (oneinit-recipes) +
      multi-registry subscriptions + 3-tier resolution
- ✅ Environment capture & migration: capture (7 language detectors) / export / import
- ✅ Clean uninstall: SQLite manifest tracks every change, full rollback
- ✅ Interactive TUI
- ✅ AI-friendly: `--json` structured output + Skill integration
- ✅ Supply-chain security: SHA256 checksum verification / `--allow-exec`
      default-deny / registry Ed25519 signing
- ✅ Team environment sync (`oneinit team`): auto-detect & sync shared dev
      env + optional signing
- ✅ Environment visualization (`oneinit viz`): ASCII tree / HTML(SVG) report /
      Issue snapshot
- ✅ Website (EN/中文) + /changelog + npm releases

## 🔜 Next (v0.2.0 target)

Ordered by contribution value; 🟢 marks newcomer-friendly entries:

| Item | Description | Status |
|------|-------------|--------|
| Apply mirrors in `oneinit sync` | local oneinit.yaml mirrors currently only log; reuse team sync's apply_mirrors | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| `oneinit cache clean` | cache/temp cleanup command (temp/ + stale cache) | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| TUI team env status | show team sync status / provide sync entry in TUI | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| Environment snapshot dotfiles | capture/export dotfile collection & restore (design doc ready) | 🟢 [Good First Issue](https://github.com/oneinitAI/oneinit/issues) |
| Tool version selection | `oneinit install python@3.11` explicit version choice (currently recipe defaults) | planned |
| `oneinit outdated` | check installed tools for newer versions | planned |
| Self-update | `oneinit self-update` (verify SHA256SUMS then upgrade) | planned |
| TUI search enhancement | filter the available list in TUI | planned |

## 🔭 Later

- **Windows native installer** (MSI) and better system integration
- **Recipe ecosystem growth**: more recipes, recipe versioning, sponsor recipes
- **Team env visualization on the website**: show team env status
- **v1.0.0 stable**: stable API, complete docs, security audit

## 🧭 How to influence the roadmap

- **Want a feature?** Open a `Feature request` Issue — describe the scenario and
  value, join the discussion.
- **Want to contribute now?** Start with the 🟢 Good First Issues — they're
  well-scoped, have clear acceptance criteria, and give you a quick win.
- **Opinions on the roadmap itself?** Open an Issue and discuss priority.

---

*The roadmap evolves with the project. Status synced as of v0.1.0-beta.2.*
