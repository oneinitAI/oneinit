# Releasing / 版本发布

> Policy enforced by the maintainer. **Always ask the user before releasing
> anything — never tag/push a release on your own.**
> 维护者策略：**每次发行前必须先征求用户确认**，绝不自行发布。

## Versioning Rules (SemVer 2.0.0)

Format: `vX.Y.Z[-beta.N]`

| Change / 变更 | Version bump / 版本号 |
|---|---|
| Breaking change (API/CLI incompatible) / 破坏性变更 | major: `v0.2.0 → v1.0.0`（v0.x 阶段 → minor，见 SemVer §5.1） |
| New feature (backward-compatible) / 新功能 | minor: `v0.2.0 → v0.3.0` |
| Bug / security fix / 修复 | patch: `v0.2.0 → v0.2.1` |

- **Prerelease counter**: `v0.3.0-beta.0 → v0.3.0-beta.1 → …`
- **Every release is a `beta` prerelease until the user declares a stable
  (正式版) release** / 用户宣布"发行正式版"之前一律使用 `-beta.N`
- **npm tag**: `beta` for prereleases; `latest` only for the stable release

## Process / 流程

1. **Ask the user first** — do not tag/push on your own / 先问用户，不自行发布
2. Bump **both** `Cargo.toml` and `npm/package.json` to the same version
   (one commit) / 同步更新两个版本号
3. `git tag vX.Y.Z-beta.N`, push → CI builds + GitHub Release + npm publish
   / 打 tag 推送，CI 自动构建/发行/npm 发布
4. Write bilingual release notes (English primary, Chinese secondary)
   / 双语发行说明（英文为主）

Source of truth / 规则原文: `开发/语义化版本规则.md` (local, not pushed to repo).
