# Contributors / 贡献者名单

> This list is **synced with GitHub** — the website
> ([oneinit.bg4jts.cn/recipes](https://oneinit.bg4jts.cn/recipes)) fetches it
> live from the GitHub contributors API (both `oneinitAI/oneinit` and
> `oneinitAI/oneinit-recipes`) plus the recipe `maintainers` in `INDEX.json`.
> No manual editing needed — contribute to any repo and you appear here.
>
> 本名单与 GitHub **自动同步**：官网（oneinit.bg4jts.cn/recipes）实时拉取
> 两个仓库的 GitHub 贡献者数据 + 配方 INDEX.json 中的维护者。无需手动维护，
> 向任一仓库贡献即可上榜。

## Code Contributors / 代码贡献者

| GitHub | Repos | Contributions |
| --- | --- | --- |
| [oneinitAI](https://github.com/oneinitAI) | oneinit, oneinit-recipes | 52 |

## Recipe Maintainers / 配方维护者

| GitHub | Recipes |
| --- | --- |
| [BG4JTS](https://github.com/BG4JTS) | dotnet8, java17, mysql8, node20, rust |

---

### How to appear here / 如何上榜

- **Code**: open a PR to [oneinitAI/oneinit](https://github.com/oneinitAI/oneinit)
  or [oneinitAI/oneinit-recipes](https://github.com/oneinitAI/oneinit-recipes)
- **Recipes**: `oneinit recipe wizard <tool>` → generate → `oneinit recipe
  contribute <file>` → upload (or open a PR manually)
- **API**: `GET https://oneinit.bg4jts.cn/api/v1/contributors`

### Admin / 管理员

Site admins can manually add contributions and tag contributors (stored in
`contributors.extra.json`, merged with the GitHub-synced data):

```bash
# 新增/修改手动贡献与标签（ADMIN_TOKEN 为 Vercel 环境变量）
curl -X POST https://oneinit.bg4jts.cn/api/v1/contributors \
  -H "Authorization: Bearer $ADMIN_TOKEN" -H "Content-Type: application/json" \
  -d '{"login":"someone","contributions":12,"tags":["core","recipes"]}'

# 移除手动条目
curl -X DELETE https://oneinit.bg4jts.cn/api/v1/contributors/someone \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

管理员可手动设置贡献数与标签（数据存 `contributors.extra.json`，与 GitHub
自动数据合并展示）。
