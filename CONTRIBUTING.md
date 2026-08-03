# Contributing Guide

> Welcome! 👋 Whether this is your first open-source contribution or you're a
> seasoned contributor, this project welcomes you. Even the smallest
> contribution — fixing a typo, improving a doc line, reporting a bug — is
> valuable.

## 📢 Communication

All project discussion happens on GitHub:

| Scenario | Where |
|----------|-------|
| Report a bug | [GitHub Issues](https://github.com/oneinitAI/oneinit/issues) (use the `Bug report` template) |
| Propose a feature | [GitHub Issues](https://github.com/oneinitAI/oneinit/issues) (use the `Feature request` template) |
| Ask questions / get help | GitHub Issues (label with `Question`) |
| Discuss code | The relevant PR's comment thread (line-by-line review) |
| Find work | Issues labeled [`good first issue`](https://github.com/oneinitAI/oneinit/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) |

> 💬 **If you're new here**: whenever you're confused — docs don't make sense,
> the process is unclear, you don't know where to start — **open an Issue and
> tell us**. It's extremely valuable to the project, and we'll improve the docs
> based on it.

---

## 🚀 Getting Started (from zero to running)

```bash
# 1. Prerequisites: Rust 1.94+ (core), Node 18+ (website only)
git clone https://github.com/oneinitAI/oneinit.git
cd oneinit

# 2. Build and test
cargo build
cargo test          # all tests must pass (currently 51)

# 3. Run it — visualize your own environment
cargo run -- viz    # see what this tool does

# 4. (Optional) website
cd www && npm install && npm run dev
```

---

## 🐛 How to Report a Bug

1. Open a [New Issue](https://github.com/oneinitAI/oneinit/issues/new/choose) and pick the **Bug report** template.
2. **Most important step**: run `oneinit viz --issue` and paste the generated
   environment snapshot into the Issue. It captures your OS, installed tools,
   and cache state in one shot — it saves a lot of back-and-forth.
3. Describe clearly:
   - The exact command you ran (full command + output)
   - What you expected to happen
   - What actually happened (error message)
4. For crashes, include the full backtrace: `RUST_BACKTRACE=1 oneinit <command>`

> Example snapshot (`oneinit viz --issue` output):
> ```
> ## Environment Snapshot
> - **OneInit**: 0.1.0-beta.2
> - **OS / Arch**: windows / x86_64
> - **~/.oneinit**: C:\Users\xxx\.oneinit (total 5.9 MB)
> ### Installed tools
> | Tool | Version | Active | Install path |
> ...
> ```

---

## 💡 How to Propose a Feature

1. Open a [New Issue](https://github.com/oneinitAI/oneinit/issues/new/choose) and pick the **Feature request** template.
2. Make two things clear:
   - **The problem you're solving** (context + pain point — more persuasive than "I want feature X")
   - **A proposed approach** (design, interface, example)
3. Check [ROADMAP.md](ROADMAP.md) first — your idea may already be planned, or
   conflict with the current direction. Discuss before coding.

---

## 🚀 How to Submit Code (PR)

### Step 1: Fork and branch

```bash
git clone https://github.com/<your-username>/oneinit.git
cd oneinit
git checkout -b feat/your-feature     # or fix/xxx, docs/xxx, security/xxx
```

### Step 2: Write code, follow the style

```bash
cargo fmt                            # formatting
cargo clippy --all-targets -- -D warnings   # zero warnings
cargo test                           # all pass
```

See [Code Style](#-code-style) below.

### Step 3: Commit (Conventional Commits)

One commit per task. Use conventional commit messages:

```
feat(viz): add HTML report output        # new feature
fix(team): resolve signature mismatch    # bug fix
security: harden checksum verification   # security fix
docs: clarify install instructions       # documentation
refactor(registry): simplify merge       # refactor
chore: bump version                      # misc
```

### Step 4: Push and open a PR

1. Push the branch: `git push origin feat/your-feature`
2. Open a PR and fill in the [PR template](.github/PULL_REQUEST_TEMPLATE.md):
   what changed, why, how you tested.
3. CI runs **tiered checks** based on change size (S/M/L):

| Check | Description |
|-------|-------------|
| Lint + Test | always: fmt + clippy + all unit tests |
| Release Build | builds release artifacts (for larger changes) |
| Docs Link Check | validates doc links |
| Workflow YAML Check | validates workflow syntax |

4. **Review**: changes to `/src`, `Cargo.toml`, `install.sh` require `@oneinitAI`
   code-owner approval. Paths **without** code owners (`www/`, docs,
   `.agents/skills/`, `.github/`) skip that approval step — **but every PR is
   still reviewed and merged by the maintainer** before it lands.
5. Merges use **squash** (clean history).

---

## 📐 Code Style

- **Formatting**: `cargo fmt` (default rustfmt config)
- **Zero warnings**: `cargo clippy --all-targets -- -D warnings` must pass
- **Error handling**: use `CoreError` + `Result` (defined in `src/core/mod.rs`);
  avoid bare `unwrap()` outside tests or provably-infallible spots
- **Comments**: **English**. Note: the existing codebase still has Chinese
  comments — we're migrating to English gradually, so **new code must use
  English comments**. Public APIs (`pub fn`) get `///` doc comments.
- **Output**: every CLI command supports both human-readable output and
  `--json` structured output (via `OutputFormatter`)
- **Dependencies**: any new dependency must be justified in the PR description
  (the project keeps a deliberately small dependency set)
- **Security**: any change touching downloads, command execution, or file
  writes must respect the project's security model (SHA256 verification,
  `--allow-exec` default-deny, signature verification) — see the README

## 🧪 Testing Requirements

- **New features must ship with tests**: unit tests live in
  `#[cfg(test)] mod tests` (follow existing patterns)
- Network-dependent logic: only test the non-network parts, or extract the
  network call for mocking
- `cargo test` must be fully green before submitting

---

## 📚 Want to contribute docs / recipes / website?

| Direction | Where |
|-----------|-------|
| Project docs | Root `*.md` files + the `docs/` folder (`docs/team-env.md`, `docs/团队环境.md`, …) |
| Community recipes | [oneinitAI/oneinit-recipes](https://github.com/oneinitAI/oneinit-recipes) (separate guide) |
| Website | `www/` in this repo (frontend only) |
| AI Skill | `.agents/skills/oneinit/SKILL.md` in this repo |

---

## ❓ Newcomer FAQ

**Q: Why does my PR say "no review required"?**
PRs touching only paths without code owners (`www/`, docs, `.agents/`,
`.github/`) skip the code-owner approval step. They still need CI to pass and
the maintainer to review/merge — so they're not merged automatically.

**Q: Some CI checks show "skipping" — does that count as passing?**
Yes. Tiered checks: small PRs only run the necessary checks; others are marked
skipping (not applicable) and don't block merging. Real failures show ❌.

**Q: A `good first issue` I want is taken?**
Comment on the Issue ("I'd like to work on this") and a maintainer will assign
it. If it's already assigned, pick another — the repo keeps 3–5 unassigned
Good First Issues.

**Q: I don't know Rust. Can I contribute?**
Yes. Docs, the website, recipes, tests, and translations are all great
starting points, and some Good First Issues don't require deep Rust.

**Q: Not sure my code is right?**
Run `cargo fmt && cargo clippy && cargo test` — green means it's mostly fine.
CI will double-check.

---

## 🗣️ Feedback is Encouraged

**You're reading this doc, which likely makes you one of the earliest
contributors.** If anything confuses you, spend a minute and tell us:

- Docs unclear? → Open an Issue, label `documentation`
- Build failing? → Open an Issue, label `bug` (attach `oneinit viz --issue` snapshot)
- Don't know what to contribute? → See [Good First Issues](https://github.com/oneinitAI/oneinit/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

> Confusion feedback from early contributors is the most valuable input for
> improving this project's docs and processes. 🙏

---

## 🧭 Roadmap

See **[ROADMAP.md](ROADMAP.md)** for project goals and direction.
