import { NextResponse } from "next/server";

/**
 * 健康检查 — oneinit.bg4jts.cn/api/v1/health
 *
 * 返回服务状态 + 上游依赖可达性（配方 INDEX、GitHub API）。
 * 全部正常 → 200 healthy；任一依赖失败 → 503 degraded。
 */

const REPO = "oneinitAI/oneinit-recipes";
const INDEX_URL = `https://raw.githubusercontent.com/${REPO}/main/INDEX.json`;
const UA = { "User-Agent": "oneinit-bg4jts-cn" };

export const dynamic = "force-dynamic";

export async function GET() {
  const started = Date.now();

  const [indexOk, githubOk] = await Promise.all([
    fetch(INDEX_URL, { signal: AbortSignal.timeout(8000) })
      .then((r) => r.ok)
      .catch(() => false),
    fetch(`https://api.github.com/repos/${REPO}`, {
      headers: UA,
      signal: AbortSignal.timeout(8000),
    })
      .then((r) => r.ok)
      .catch(() => false),
  ]);

  const dependencies = {
    recipes_index: indexOk,
    github_api: githubOk,
  };
  const healthy = indexOk && githubOk;

  return NextResponse.json(
    {
      ok: healthy,
      service: "oneinit-bg4jts-cn",
      status: healthy ? "healthy" : "degraded",
      time: new Date().toISOString(),
      latency_ms: Date.now() - started,
      dependencies,
    },
    { status: healthy ? 200 : 503 }
  );
}
