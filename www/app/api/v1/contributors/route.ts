import { NextResponse } from "next/server";

/**
 * 贡献者名单 — oneinit.bg4jts.cn/api/v1/contributors
 *
 * 数据与 GitHub 同步（单一数据源）：
 *   - 两个仓库的 GitHub contributors API（代码贡献）
 *   - 配方仓库 INDEX.json 中的 maintainers（配方贡献）
 *   - contributors.extra.json（oneinit 仓库）— 管理员手动设置的
 *     贡献数与标签（manual）
 * 按登录名合并、去重，按贡献数排序。
 *
 * GET 缓存 5 分钟（每 5 分钟刷新，管理员修改后最多 5 分钟生效）；
 * POST（管理员）写入 extra 文件后即时生效。
 */

const MAIN_REPO = "oneinitAI/oneinit";
const REPO_INDEX = "oneinitAI/oneinit-recipes";
const REPOS = [MAIN_REPO, REPO_INDEX];
const GH = "https://api.github.com";
const INDEX_URL = `https://raw.githubusercontent.com/${REPO_INDEX}/main/INDEX.json`;
const EXTRA_PATH = "contributors.extra.json";
const EXTRA_RAW_URL = `https://raw.githubusercontent.com/${MAIN_REPO}/main/${EXTRA_PATH}`;
const EXTRA_API_URL = `${GH}/repos/${MAIN_REPO}/contents/${EXTRA_PATH}`;
const UA = { "User-Agent": "oneinit-bg4jts-cn" };

/** GitHub API 读取统一带 GITHUB_TOKEN 认证（避免未认证限流 60/时） */
function ghAuth(): Record<string, string> {
  const token = process.env.GITHUB_TOKEN;
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export type Contributor = {
  login: string;
  html_url: string;
  avatar_url: string;
  contributions: number;
  repos: string[];
  source: string[];
  tags: string[];
};

type ExtraEntry = { login: string; contributions?: number; tags?: string[] };
type ExtraFile = { entries: ExtraEntry[] };

/** 读取手动贡献/标签文件（GET 用 raw.githubusercontent，无 API 限流）；
 *  不存在/失败时降级为空文件 */
async function readExtraRaw(): Promise<ExtraFile> {
  try {
    const res = await fetch(EXTRA_RAW_URL, { next: { revalidate: 300 } });
    if (res.status === 404) return { entries: [] };
    if (!res.ok) return { entries: [] };
    return (await res.json()) as ExtraFile;
  } catch {
    return { entries: [] };
  }
}

/** 读取 extra 文件 + sha（管理员写入用 contents API，已认证） */
async function readExtraWithSha(): Promise<{ data: ExtraFile; sha?: string }> {
  const res = await fetch(EXTRA_API_URL, { headers: { ...UA, ...ghAuth() } });
  if (res.status === 404) return { data: { entries: [] } };
  if (!res.ok) throw new Error(`read extra file failed (${res.status})`);
  const j = await res.json();
  const content = Buffer.from(j.content, "base64").toString("utf-8");
  const data = JSON.parse(content) as ExtraFile;
  return { data, sha: j.sha };
}

/** 把手动数据合并进贡献者表（标签 / 手动贡献数） */
function mergeExtra(merged: Record<string, Contributor>, extra: ExtraFile) {
  for (const e of extra.entries || []) {
    const c = merged[e.login];
    if (c) {
      if (e.contributions != null) c.contributions = e.contributions;
      if (e.tags?.length) {
        for (const t of e.tags) if (!c.tags.includes(t)) c.tags.push(t);
      }
      if (!c.source.includes("manual")) c.source.push("manual");
    } else {
      merged[e.login] = {
        login: e.login,
        html_url: `https://github.com/${e.login}`,
        avatar_url: avatarUrl(e.login),
        contributions: e.contributions ?? 1,
        repos: [],
        source: ["manual"],
        tags: e.tags ?? [],
      };
    }
  }
}

/** 头像统一走 avatars.githubusercontent.com（免重定向；github.com/{login}.png
 *  在账号被禁用时会 404，而该 CDN 仍可访问） */
function avatarUrl(login: string): string {
  return `https://avatars.githubusercontent.com/${login}?v=4`;
}

/** 校验管理员令牌（ADMIN_TOKEN） */
function isAdmin(req: Request): boolean {
  const token = process.env.ADMIN_TOKEN;
  if (!token) return false;
  const auth = req.headers.get("authorization") || "";
  return auth === `Bearer ${token}`;
}

export const revalidate = 300;

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
      tags: [],
    };
    c.contributions += n;
    if (!c.repos.includes(repo)) c.repos.push(repo);
    if (!c.source.includes(source)) c.source.push(source);
    merged[login] = c;
  };

  // 1. GitHub contributors（两个仓库，带 token 认证避免限流）
  await Promise.all(
    REPOS.map(async (repo) => {
      try {
        const res = await fetch(`${GH}/repos/${repo}/contributors?per_page=100`, {
          headers: { ...UA, ...ghAuth() },
          next: { revalidate: 300 },
        });
        if (!res.ok) return;
        const list: any[] = await res.json();
        for (const c of list) {
          upsert(c.login, c.html_url, c.avatar_url, c.contributions || 0, repo.split("/")[1], "github");
        }
      } catch {
        /* 单个仓库失败不影响整体 */
      }
    })
  );

  // 2. 配方仓库 INDEX maintainers（配方贡献者）
  try {
    const idxRes = await fetch(INDEX_URL, { next: { revalidate: 300 } });
    if (idxRes.ok) {
      const index = await idxRes.json();
      for (const p of Object.values(index.packages || {}) as any[]) {
        for (const m of p.maintainers || []) {
          upsert(m, `https://github.com/${m}`, avatarUrl(m), 1, "oneinit-recipes", "maintainer");
        }
      }
    }
  } catch {
    /* 忽略 */
  }

  // 3. 手动贡献 + 标签（contributors.extra.json，走 raw 无 API 限流）
  const extra = await readExtraRaw();
  mergeExtra(merged, extra);

  const contributors = Object.values(merged).sort((a, b) => b.contributions - a.contributions);

  return NextResponse.json({
    ok: true,
    total: contributors.length,
    contributors,
  });
}

// POST /api/v1/contributors — 管理员：新增/修改手动贡献与标签
export async function POST(req: Request) {
  if (!isAdmin(req)) {
    return NextResponse.json({ ok: false, error: "unauthorized (invalid ADMIN_TOKEN)" }, { status: 401 });
  }

  let body: ExtraEntry;
  try {
    body = await req.json();
  } catch {
    return NextResponse.json({ ok: false, error: "invalid JSON body" }, { status: 400 });
  }
  const login = body?.login;
  if (!login || typeof login !== "string" || !/^[a-zA-Z0-9-]+$/.test(login)) {
    return NextResponse.json({ ok: false, error: "login must be a GitHub username" }, { status: 400 });
  }
  if (body.contributions != null && (typeof body.contributions !== "number" || body.contributions < 0)) {
    return NextResponse.json({ ok: false, error: "contributions must be a non-negative number" }, { status: 400 });
  }
  const tags = Array.isArray(body.tags) ? body.tags.map(String).slice(0, 8) : undefined;

  const token = process.env.GITHUB_TOKEN;
  if (!token) {
    return NextResponse.json({ ok: false, error: "admin write not configured (missing GITHUB_TOKEN)" }, { status: 503 });
  }

  try {
    const { data, sha } = await readExtraWithSha();
    const entries = data.entries.filter((e) => e.login !== login);
    entries.push({ login, ...(body.contributions != null ? { contributions: body.contributions } : {}), ...(tags ? { tags } : {}) });
    const next: ExtraFile = { entries };
    await writeExtra(token, next, sha);
    return NextResponse.json({ ok: true, login, contributions: body.contributions ?? null, tags: tags ?? [] });
  } catch (e: any) {
    return NextResponse.json({ ok: false, error: `github: ${e.message || "unknown"}` }, { status: 502 });
  }
}

/** 写回 contributors.extra.json（直接提交 main，管理员操作） */
async function writeExtra(token: string, data: ExtraFile, sha?: string) {
  const content = Buffer.from(JSON.stringify(data, null, 2) + "\n").toString("base64");
  const res = await fetch(EXTRA_API_URL, {
    method: "PUT",
    headers: { ...UA, Authorization: `Bearer ${token}` },
    body: JSON.stringify({
      message: "contributors: update manual entries/tags (admin)",
      content,
      ...(sha ? { sha } : {}),
      branch: "main",
    }),
  });
  if (!res.ok) {
    const errText = await res.text().catch(() => "");
    throw new Error(`PUT contents failed (${res.status}) ${errText.slice(0, 200)}`);
  }
}
