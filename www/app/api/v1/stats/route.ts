import { NextResponse } from "next/server";

/**
 * 配方统计 — oneinit.bg4jts.cn/api/v1/stats
 *
 * 聚合「GitHub 配方仓库（已合并配方）」+「上传的配方（open PRs 待审核）」
 * 的统计数据，供前端展示页使用。结果缓存 5 分钟。
 */

const REPO = "oneinitAI/oneinit-recipes";
const GH = "https://api.github.com";
const INDEX_URL = `https://raw.githubusercontent.com/${REPO}/main/INDEX.json`;

const UA = { "User-Agent": "oneinit-bg4jts-cn" };

/** GitHub API 读取统一带 GITHUB_TOKEN 认证（避免未认证限流 60/时） */
function ghAuth(): Record<string, string> {
  const token = process.env.GITHUB_TOKEN;
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export const revalidate = 300;

export async function GET() {
  try {
    // 1. INDEX.json — 已合并配方的权威清单
    const indexRes = await fetch(INDEX_URL, { next: { revalidate: 300 } });
    const index = indexRes.ok ? await indexRes.json() : null;
    const packages: Record<string, any> = index?.packages || {};

    // 2. 平台覆盖 — recipes/<name>/<version>.yaml（限量 60 个配方）
    const platformCoverage = { windows: 0, linux: 0, darwin: 0 };
    const listRes = await fetch(`${GH}/repos/${REPO}/contents/recipes`, { headers: { ...UA, ...ghAuth() }, next: { revalidate: 300 } });
    if (listRes.ok) {
      const dirs: any[] = await listRes.json();
      const recipeDirs = dirs.filter((d) => d.type === "dir").slice(0, 60);
      await Promise.all(
        recipeDirs.map(async (d) => {
          try {
            const innerRes = await fetch(d.url, { headers: { ...UA, ...ghAuth() }, next: { revalidate: 300 } });
            if (!innerRes.ok) return;
            const inner: any[] = await innerRes.json();
            const yamlFile = inner.find((f) => f.name.endsWith(".yaml"));
            if (!yamlFile) return;
            const r = await fetch(yamlFile.download_url, { next: { revalidate: 300 } });
            if (!r.ok) return;
            const txt = await r.text();
            if (/\n\s*windows:/.test(txt)) platformCoverage.windows++;
            if (/\n\s*linux:/.test(txt)) platformCoverage.linux++;
            if (/\n\s*darwin:/.test(txt)) platformCoverage.darwin++;
          } catch {
            /* 单个失败不影响整体 */
          }
        })
      );
    }

    // 3. 待审核上传（open PRs）
    const prRes = await fetch(`${GH}/repos/${REPO}/pulls?state=open&per_page=50`, {
      headers: UA,
      next: { revalidate: 300 },
    });
    const prs: any[] = prRes.ok ? await prRes.json() : [];
    const pendingUploads = prs.map((p) => ({
      number: p.number,
      title: p.title,
      url: p.html_url,
      author: p.user?.login || "unknown",
      created_at: p.created_at,
    }));

    // 4. 聚合：按标签 / 维护者
    const tagCount: Record<string, number> = {};
    const maintainerCount: Record<string, number> = {};
    for (const p of Object.values(packages) as any[]) {
      for (const tag of p.tags || []) tagCount[tag] = (tagCount[tag] || 0) + 1;
      for (const m of p.maintainers || []) maintainerCount[m] = (maintainerCount[m] || 0) + 1;
    }
    const topTags = Object.entries(tagCount)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 12)
      .map(([tag, count]) => ({ tag, count }));
    const topMaintainers = Object.entries(maintainerCount)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(([name, count]) => ({ name, count }));

    // 5. 配方目录
    const recipes = Object.entries(packages).map(([name, e]: [string, any]) => ({
      name,
      description: e.description || "",
      latest: e.latest,
      versions: e.versions || [],
      tags: e.tags || [],
      maintainers: e.maintainers || [],
    }));

    return NextResponse.json({
      ok: true,
      total_recipes: recipes.length,
      pending_uploads: pendingUploads.length,
      last_updated: index?.last_updated || null,
      top_tags: topTags,
      top_maintainers: topMaintainers,
      platform_coverage: platformCoverage,
      pending_uploads_list: pendingUploads,
      recipes,
    });
  } catch (e: any) {
    return NextResponse.json({ ok: false, error: e.message }, { status: 500 });
  }
}
