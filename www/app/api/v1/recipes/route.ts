import { NextResponse } from "next/server";
import YAML from "yaml";

/**
 * 配方贡献后端 — oneinit.bg4jts.cn/api/v1/recipes
 *
 * - POST: 接收 oneinit 上传的配方 YAML，校验后通过 GitHub API 创建 PR
 *   到 oneinitAI/oneinit-recipes（复用仓库现有的 validate + sign 管道）。
 * - GET: 返回配方目录（INDEX.json）+ 待审核上传（open PRs）。
 *
 * 需要 Vercel 环境变量 GITHUB_TOKEN（带 repo 权限的 PAT / 机器人 token）。
 */

const REPO = "oneinitAI/oneinit-recipes";
const GH = "https://api.github.com";
const INDEX_URL = `https://raw.githubusercontent.com/${REPO}/main/INDEX.json`;

function ghHeaders(token?: string) {
  return {
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
    "User-Agent": "oneinit-bg4jts-cn",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}

const VALID_INSTALL_TYPES = [
  "zip_extract",
  "tar_extract",
  "exe_silent",
  "binary_copy",
  "msi_install",
  "pkg_install",
];

/** 校验配方：name 合法 + 至少一个平台带 url + install_type 合法 */
function validateRecipe(recipe: any): string | null {
  if (!recipe || typeof recipe.name !== "string") {
    return "recipe must have a `name` field";
  }
  if (!/^[a-z0-9][a-z0-9._-]*$/.test(recipe.name)) {
    return "name must match [a-z0-9][a-z0-9._-]* (lowercase letters, digits, . _ -)";
  }
  const platforms = recipe.platforms || {};
  const cfgs = Object.values(platforms) as any[];
  const hasUrl = cfgs.some(
    (p) => p && typeof p.url === "string" && p.url.startsWith("http") && p.url.length > 12
  );
  if (!hasUrl) {
    return "recipe must have at least one platform with a valid http(s) url";
  }
  for (const p of cfgs) {
    if (p && p.install_type && !VALID_INSTALL_TYPES.includes(p.install_type)) {
      return `invalid install_type: ${p.install_type}`;
    }
  }
  return null;
}

/** 通过 GitHub API 创建「新增配方」的 PR */
async function createRecipePullRequest(token: string, name: string, version: string, yamlText: string) {
  const branch = `upload/${name}-${Date.now().toString(36)}`;
  // 仓库布局约定：recipes/<name>/<version>.yaml（validate.py 强制）
  const path = `recipes/${name}/${version}.yaml`;
  const headers = ghHeaders(token);

  // 1. main 引用 → 2. 建分支
  const refRes = await fetch(`${GH}/repos/${REPO}/git/ref/heads/main`, { headers });
  if (!refRes.ok) throw new Error(`cannot read main ref (${refRes.status})`);
  const ref = await refRes.json();
  const baseSha = ref.object.sha;

  const branchRes = await fetch(`${GH}/repos/${REPO}/git/refs`, {
    method: "POST",
    headers,
    body: JSON.stringify({ ref: `refs/heads/${branch}`, sha: baseSha }),
  });
  if (!branchRes.ok) throw new Error(`cannot create branch (${branchRes.status})`);

  // 3. blob
  const blobRes = await fetch(`${GH}/repos/${REPO}/git/blobs`, {
    method: "POST",
    headers,
    body: JSON.stringify({ content: yamlText, encoding: "utf-8" }),
  });
  if (!blobRes.ok) throw new Error(`cannot create blob (${blobRes.status})`);
  const blob = await blobRes.json();

  // 4. tree
  const treeRes = await fetch(`${GH}/repos/${REPO}/git/trees`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      base_tree: baseSha,
      tree: [{ path, mode: "100644", type: "blob", sha: blob.sha }],
    }),
  });
  if (!treeRes.ok) throw new Error(`cannot create tree (${treeRes.status})`);
  const tree = await treeRes.json();

  // 5. commit
  const commitRes = await fetch(`${GH}/repos/${REPO}/git/commits`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      message: `add ${name} recipe (uploaded via oneinit.bg4jts.cn)`,
      tree: tree.sha,
      parents: [baseSha],
    }),
  });
  if (!commitRes.ok) throw new Error(`cannot create commit (${commitRes.status})`);
  const commit = await commitRes.json();

  // 6. 更新分支
  const updateRes = await fetch(`${GH}/repos/${REPO}/git/refs/heads/${branch}`, {
    method: "PATCH",
    headers,
    body: JSON.stringify({ sha: commit.sha, force: false }),
  });
  if (!updateRes.ok) throw new Error(`cannot update branch (${updateRes.status})`);

  // 7. PR
  const prRes = await fetch(`${GH}/repos/${REPO}/pulls`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      title: `add ${name} recipe (uploaded via oneinit.bg4jts.cn)`,
      head: branch,
      base: "main",
      body: [
        "Recipe uploaded from OneInit via oneinit.bg4jts.cn.",
        "",
        "The repository's validate + sign workflow handles validation and",
        "INDEX.json signing after merge.",
      ].join("\n"),
    }),
  });
  if (!prRes.ok) throw new Error(`cannot create pull request (${prRes.status})`);
  const pr = await prRes.json();

  return { branch, pull_request_url: pr.html_url, pull_request_number: pr.number };
}

export const dynamic = "force-dynamic";

// POST /api/v1/recipes — 上传配方
export async function POST(req: Request) {
  const raw = await req.text();
  let recipe: any;
  try {
    recipe = YAML.parse(raw);
  } catch {
    return NextResponse.json({ ok: false, error: "invalid YAML" }, { status: 400 });
  }

  const err = validateRecipe(recipe);
  if (err) {
    return NextResponse.json({ ok: false, error: err }, { status: 400 });
  }

  const version = String(recipe.version ?? "1.0.0").replace(/[^a-z0-9._+-]/g, "");
  if (!version) {
    return NextResponse.json({ ok: false, error: "recipe must have a `version` field" }, { status: 400 });
  }

  const token = process.env.GITHUB_TOKEN;
  if (!token) {
    return NextResponse.json(
      { ok: false, error: "upload service is not configured (missing GITHUB_TOKEN)" },
      { status: 503 }
    );
  }

  try {
    const result = await createRecipePullRequest(token, recipe.name, version, raw);
    return NextResponse.json({ ok: true, ...result });
  } catch (e: any) {
    return NextResponse.json(
      { ok: false, error: `github: ${e.message || "unknown"}` },
      { status: 502 }
    );
  }
}

export const revalidate = 300;

// GET /api/v1/recipes — 配方目录 + 待审核上传
export async function GET() {
  try {
    const indexRes = await fetch(INDEX_URL, { next: { revalidate: 300 } });
    const index = indexRes.ok ? await indexRes.json() : null;
    const packages: Record<string, any> = index?.packages || {};

    const prRes = await fetch(`${GH}/repos/${REPO}/pulls?state=open&per_page=50`, {
      headers: ghHeaders(process.env.GITHUB_TOKEN),
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
      total: recipes.length,
      last_updated: index?.last_updated || null,
      recipes,
      pending_uploads: pendingUploads,
    });
  } catch (e: any) {
    return NextResponse.json({ ok: false, error: e.message }, { status: 500 });
  }
}
