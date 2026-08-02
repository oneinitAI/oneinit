# 贡献指南 / Contributing Guide

> 欢迎！👋 无论你是第一次接触开源，还是经验丰富的贡献者，这个项目都欢迎你。
> 即使是最小的贡献（修一个错别字、补一条文档、报告一个 bug）都很有价值。

## 📢 沟通平台（Communication）

项目的一切讨论都在 GitHub 上进行：

| 场景 | 去哪里 |
|------|--------|
| 报告 Bug | [GitHub Issues](https://github.com/oneinitAI/oneinit/issues)（选 `Bug report` 模板） |
| 提议新功能 | [GitHub Issues](https://github.com/oneinitAI/oneinit/issues)（选 `Feature request` 模板） |
| 提问 / 求助 | GitHub Issues（选 `Question` 标签） |
| 讨论代码 | 对应 PR 的评论区（review 时逐行讨论） |
| 找活干 | 标了 [`good first issue`](https://github.com/oneinitAI/oneinit/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) 的 Issue |

> 💬 **如果你是新人**：遇到任何困惑——文档看不懂、流程不清楚、不知道从哪下手——**直接开一个 Issue 告诉我们**。这对项目非常宝贵，我们会据此改进文档。

---

## 🚀 快速开始（从 0 到能跑）

```bash
# 1. 环境要求：Rust 1.94+（构建核心）、Node 18+（仅官网需要）
git clone https://github.com/oneinitAI/oneinit.git
cd oneinit

# 2. 构建 + 测试
cargo build
cargo test          # 全部测试必须通过（当前 51 个）

# 3. 跑起来看看你的环境
cargo run -- viz    # 环境可视化——看看这个工具能做什么

# 4.（可选）官网
cd www && npm install && npm run dev
```

---

## 🐛 如何报告 Bug

1. 打开 [New Issue](https://github.com/oneinitAI/oneinit/issues/new/choose)，选择 **Bug report** 模板
2. **最重要的一个步骤**：运行 `oneinit viz --issue`，把生成的环境快照直接粘贴进 Issue。
   这一张快照包含了你的系统、已装工具、缓存状态，能省掉大量来回确认的时间。
3. 描述清楚：
   - 你执行了什么命令（完整命令 + 输出）
   - 期望发生什么
   - 实际发生了什么（报错信息）
4. 如果是崩溃，附上 `RUST_BACKTRACE=1 oneinit <命令>` 的完整回溯

> 环境快照示例（`oneinit viz --issue` 的输出）：
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

## 💡 如何提议新功能

1. 打开 [New Issue](https://github.com/oneinitAI/oneinit/issues/new/choose)，选择 **Feature request** 模板
2. 说清楚两件事：
   - **要解决什么问题**（场景 + 痛点，比"我想要 X 功能"更有说服力）
   - **建议怎么做**（方案、接口、示例）
3. 先看看 [ROADMAP.md](ROADMAP.md) —— 你的想法可能已经在规划中，或与现有方向冲突，先讨论再动手

---

## 🚀 如何提交代码（PR）

### 第一步：Fork 并建分支

```bash
git clone https://github.com/<你的用户名>/oneinit.git
cd oneinit
git checkout -b feat/你的功能名     # 或 fix/xxx、docs/xxx、security/xxx
```

### 第二步：写代码，遵守规范

```bash
cargo fmt                            # 格式化
cargo clippy --all-targets -- -D warnings   # 零警告
cargo test                           # 全部通过
```

规范细节见下方 [代码风格规范](#-代码风格规范)。

### 第三步：提交（Conventional Commits）

一次提交只做一件事，提交信息用约定式提交：

```
feat(viz): add HTML report output        # 新功能
fix(team): resolve signature mismatch    # 修复
security: harden checksum verification   # 安全修复
docs: clarify install instructions       # 文档
refactor(registry): simplify merge       # 重构
chore: bump version                      # 杂项
```

### 第四步：推送并开 PR

1. 推送分支：`git push origin feat/你的功能名`
2. 打开 PR，填写 [PR 模板](.github/PULL_REQUEST_TEMPLATE.md)：改了什么、为什么、怎么测
3. CI 会自动按**改动体量分级**运行检查（S/M/L）：

| 检查 | 说明 |
|------|------|
| Lint + Test | 必跑：fmt + clippy + 全部单元测试 |
| Release Build | 编译 release 产物（改动较大时跑） |
| Docs Link Check | 文档链接检查 |
| Workflow YAML Check | 工作流语法检查 |

4. **代码所有者审核**：改动 `/src`、`Cargo.toml`、`install.sh` 需要 `@oneinitAI` 审核批准；
   `www/`、`.agents/skills/`、`README*.md`、`.github/` 等路径**自动免审**（仅需 CI 通过）
5. 合并采用 **squash**（历史干净）

---

## 📐 代码风格规范

- **格式化**：`cargo fmt`（rustfmt 默认配置）
- **零警告**：`cargo clippy --all-targets -- -D warnings` 必须通过
- **错误处理**：统一用 `CoreError` + `Result`（定义在 `src/core/mod.rs`）；不要到处 `unwrap()`，
  除非在测试里或确实不可能失败的地方
- **注释**：与现有代码一致，用**中文注释**；公共 API（`pub fn`）加 `///` 文档注释
- **输出**：所有 CLI 命令同时支持人类可读输出 + `--json` 结构化输出（`OutputFormatter`）
- **依赖**：新增依赖必须在 PR 描述里说明理由（项目目前依赖很少，很在意这一点）
- **安全**：任何涉及下载、执行命令、写文件的改动，都要遵守项目现有的安全模型
  （SHA256 校验、`--allow-exec` 默认拒绝、签名验证），见 [安全与免责声明](README.md#security--disclaimer)

## 🧪 测试要求

- **新功能必须带测试**：单元测试放在 `#[cfg(test)] mod tests` 中（跟随现有模式）
- 涉及网络的逻辑只测非网络部分，或把网络调用抽出来 mock
- `cargo test` 必须全绿再提交

---

## 📚 想贡献文档 / 配方 / 官网？

| 方向 | 去哪里 |
|------|--------|
| 项目文档 | 本仓库根目录的 `*.md`（`社区配方.md`、`团队环境.md`、`开发.md` 等） |
| 社区配方 | [oneinitAI/oneinit-recipes](https://github.com/oneinitAI/oneinit-recipes)（有独立的贡献指南） |
| 官网 | 本仓库 `www/` 目录（纯前端，**免审**，CI 通过即可合并） |
| AI Skill | 本仓库 `.agents/skills/oneinit/SKILL.md`（**免审**） |

---

## ❓ 新人常见困惑（FAQ）

**Q：我的 PR 为什么显示"免审"？**
改动只涉及 `www/`、文档、`.agents/`、`.github/` 等没有代码所有者的路径时，
不需要人工审核，CI 通过即可合并。核心 Rust 代码（`/src` 等）需要审核。

**Q：CI 里有些检查显示 "skipping"，这算通过吗？**
算。分级检查机制：改动小的 PR 只跑必要的检查，其他标记为 skipping（不适用），
不影响合并。真正的失败会显示 ❌。

**Q：`good first issue` 被认领了怎么办？**
在 Issue 里评论认领（"I'd like to work on this"），维护者会分配给你。
如果被标记为已认领/有 assignee，就换一个 —— 仓库会保持 3-5 个未分配的
Good First Issues。

**Q：我不会 Rust，能贡献吗？**
能。文档、官网、配方、测试、翻译都是很好的起点。Good First Issues 里也有
不需要深入 Rust 的任务。

**Q：改完代码不知道对不对？**
先跑 `cargo fmt && cargo clippy && cargo test`，全过就基本没问题。
CI 也会帮你检查。

---

## 🗣️ 鼓励反馈

**你正在读这份文档，说明你可能是最早期的贡献者之一。**
如果你遇到任何困惑，请花一分钟告诉我们：

- 文档哪里看不懂？→ 开 Issue 标 `documentation`
- 构建失败了？→ 开 Issue 标 `bug`（附 `oneinit viz --issue` 快照）
- 不知道贡献什么？→ 看 [Good First Issues](https://github.com/oneinitAI/oneinit/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

> 早期贡献者的困惑反馈，是项目改进文档和流程的最宝贵输入。🙏

---

## 🧭 路线图

项目的目标和发展方向见 **[ROADMAP.md](ROADMAP.md)**。
