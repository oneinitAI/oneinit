import { NextResponse } from "next/server";

/**
 * 贡献者名单 — oneinit.bg4jts.cn/api/v1/contributors
 *
 * 数据与 GitHub 同步（单一数据源）：
 *   - 两个仓库的 GitHub contributors API（代码贡献）
 *   - 配方仓库 INDEX.json 中的 maintainers（配方贡献）
 * 按登录名合并、去重，按贡献数排序。缓存 1 小时。
 */

const REPO_INDEX = "oneinitAI/oneinit-recipes";
const REPOS = ["oneinitAI/oneinit", "oneinitAI/oneinit-recipes"];
const GH = "https://api.github.com";
const INDEX_URL = `https://raw.githubusercontent.com/${REPO_INDEX}/main/INDEX.json`;
const UA = { "User-Agent": "oneinit-bg4jts-cn" };

type Contributor = {
  login: string;
  html_url: string;
  avatar_url: string;
  contributions: number;
  repos: string[];
  source: string[];
};

export const revalidate = 3600;

export async function GET() {
  const merged: Record<string, Contributor> = {};

  const upsert = (login: string, html: string, avatar: string, n: number, repo: string, source: string) => {
    const c = merged[login] ?? {
      login,
      html_url: html,
      avatar_url: avatar,
      contributions: 0,
      repos: [],
      source: [],
    };
    c.contributions += n;
    if (!c.repos.includes(repo)) c.repos.push(repo);
    if (!c.source.includes(source)) c.source.push(source);
    merged[login] = c;
  };

  // 1. GitHub contributors（两个仓库）
  await Promise.all(
    REPOS.map(async (repo) => {
      try {
        const res = await fetch(`${GH}/repos/${repo}/contributors?per_page=100`, {
          headers: UA,
          next: { revalidate: 3600 },
        });
        if (!res.ok) return;
        const list: any[] = await res.json();
        for (const c of list) {
          upsert(
            c.login,
            c.html_url,
            c.avatar_url,
            c.contributions || 0,
            repo.split("/")[1],
            "github"
          );
        }
      } catch {
        /* 单个仓库失败不影响整体 */
      }
    })
  );

  // 2. 配方仓库 INDEX maintainers（配方贡献者）
  try {
    const idxRes = await fetch(INDEX_URL, { next: { revalidate: 3600 } });
    if (idxRes.ok) {
      const index = await idxRes.json();
      for (const p of Object.values(index.packages || {}) as any[]) {
        for (const m of p.maintainers || []) {
          upsert(m, `https://github.com/${m}`, `https://github.com/${m}.png`, 1, "oneinit-recipes", "maintainer");
        }
      }
    }
  } catch {
    /* 忽略 */
  }

  const contributors = Object.values(merged)
    .sort((a, b) => b.contributions - a.contributions)
    .map((c) => ({ ...c }));

  return NextResponse.json({
    ok: true,
    total: contributors.length,
    contributors,
  });
}
