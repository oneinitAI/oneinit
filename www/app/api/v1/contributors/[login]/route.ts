import { NextResponse } from "next/server";

/**
 * DELETE /api/v1/contributors/[login] — 管理员：移除手动贡献条目
 * （contributors.extra.json 中的该登录名条目；自动同步的 GitHub 数据
 * 不受影响）。
 */

const MAIN_REPO = "oneinitAI/oneinit";
const GH = "https://api.github.com";
const EXTRA_PATH = "contributors.extra.json";
const EXTRA_URL = `${GH}/repos/${MAIN_REPO}/contents/${EXTRA_PATH}`;
const UA = { "User-Agent": "oneinit-bg4jts-cn" };

function isAdmin(req: Request): boolean {
  const token = process.env.ADMIN_TOKEN;
  if (!token) return false;
  const auth = req.headers.get("authorization") || "";
  return auth === `Bearer ${token}`;
}

export async function DELETE(req: Request, { params }: { params: Promise<{ login: string }> }) {
  if (!isAdmin(req)) {
    return NextResponse.json({ ok: false, error: "unauthorized (invalid ADMIN_TOKEN)" }, { status: 401 });
  }
  const token = process.env.GITHUB_TOKEN;
  if (!token) {
    return NextResponse.json({ ok: false, error: "admin write not configured (missing GITHUB_TOKEN)" }, { status: 503 });
  }

  const { login } = await params;
  if (!login || !/^[a-zA-Z0-9-]+$/.test(login)) {
    return NextResponse.json({ ok: false, error: "invalid login" }, { status: 400 });
  }

  try {
    const res = await fetch(EXTRA_URL, { headers: UA, next: { revalidate: 0 } });
    if (res.status === 404) {
      return NextResponse.json({ ok: true, removed: false, login });
    }
    if (!res.ok) throw new Error(`read extra file failed (${res.status})`);
    const j = await res.json();
    const data = JSON.parse(Buffer.from(j.content, "base64").toString("utf-8")) as {
      entries: { login: string }[];
    };
    const before = data.entries.length;
    data.entries = data.entries.filter((e) => e.login !== login);
    if (data.entries.length === before) {
      return NextResponse.json({ ok: true, removed: false, login });
    }

    const content = Buffer.from(JSON.stringify(data, null, 2) + "\n").toString("base64");
    const putRes = await fetch(EXTRA_URL, {
      method: "PUT",
      headers: { ...UA, Authorization: `Bearer ${token}` },
      body: JSON.stringify({
        message: `contributors: remove manual entry ${login} (admin)`,
        content,
        sha: j.sha,
        branch: "main",
      }),
    });
    if (!putRes.ok) {
      const errText = await putRes.text().catch(() => "");
      throw new Error(`PUT contents failed (${putRes.status}) ${errText.slice(0, 200)}`);
    }
    return NextResponse.json({ ok: true, removed: true, login });
  } catch (e: any) {
    return NextResponse.json({ ok: false, error: `github: ${e.message || "unknown"}` }, { status: 502 });
  }
}
