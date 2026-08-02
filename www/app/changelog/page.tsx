"use client";
import { useEffect, useState } from "react";
import { useLang } from "@/components/lang-provider";
import { Nav } from "@/components/nav";

const REPO = "oneinitAI/oneinit";
const API = `https://api.github.com/repos/${REPO}/releases`;

type Asset = {
  name: string;
  browser_download_url: string;
  size: number;
};

type Release = {
  tag_name: string;
  name: string;
  published_at: string;
  created_at: string;
  prerelease: boolean;
  body: string;
  html_url: string;
  author: { login: string };
  assets: Asset[];
};

export default function ChangelogPage() {
  const { t } = useLang();
  const [releases, setReleases] = useState<Release[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    fetch(API)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((data: Release[]) => setReleases(data || []))
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  // 简易 markdown 渲染（标题 / 列表 / 代码 / 链接）
  const renderMd = (body: string) => {
    if (!body) return null;
    return body.split("\n").map((line, i) => {
      const trimmed = line.trim();
      if (trimmed.startsWith("### ")) {
        return (
          <h3 key={i} className="mb-1 mt-4 font-mono text-sm font-bold text-white">
            {trimmed.slice(4)}
          </h3>
        );
      }
      if (trimmed.startsWith("## ")) {
        return (
          <h2 key={i} className="mb-2 mt-5 font-mono text-base font-bold text-emerald-400">
            {trimmed.slice(3)}
          </h2>
        );
      }
      if (trimmed.startsWith("- ")) {
        return (
          <li key={i} className="ml-4 list-disc text-sm leading-relaxed text-zinc-400">
            {trimmed.slice(2)}
          </li>
        );
      }
      if (trimmed.startsWith("```") || trimmed.endsWith("```")) {
        return <div key={i} className="h-0" />;
      }
      if (trimmed.startsWith("`")) {
        return (
          <div key={i} className="ml-4 text-sm text-zinc-400">
            <code className="rounded bg-zinc-800/80 px-1.5 py-0.5 font-mono text-xs text-emerald-300">
              {trimmed.replace(/`/g, "")}
            </code>
          </div>
        );
      }
      if (trimmed.startsWith("[")) {
        const m = trimmed.match(/\[(.*?)\]\((.*?)\)/);
        if (m) {
          return (
            <div key={i} className="ml-4 text-sm">
              <a href={m[2]} target="_blank" rel="noopener noreferrer" className="text-emerald-400 hover:text-emerald-300">
                {m[1]}
              </a>
            </div>
          );
        }
      }
      if (trimmed === "") return <div key={i} className="h-1.5" />;
      return (
        <p key={i} className="text-sm leading-relaxed text-zinc-400">
          {trimmed}
        </p>
      );
    });
  };

  const fmtSize = (n: number) => {
    if (n > 1048576) return `${(n / 1048576).toFixed(1)} MB`;
    return `${(n / 1024).toFixed(0)} KB`;
  };

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
      <Nav />

      <div className="mx-auto max-w-[760px] px-6 py-24">
        <a href="/" className="inline-flex items-center gap-2 font-mono text-sm text-emerald-500 hover:text-emerald-400 transition-colors">
          ← {t("terms.back")}
        </a>

        <h1 className="mt-6 text-4xl font-bold tracking-tight text-white">{t("cl.title")}</h1>
        <p className="mt-2 text-zinc-500">{t("cl.subtitle")}</p>

        <div className="mt-4 flex flex-wrap items-center gap-2">
          {[
            { href: "https://www.npmjs.com/package/oneinit", label: t("cl.npm") },
            { href: "https://github.com/oneinitAI/oneinit/releases", label: t("cl.releases") },
            { href: "https://oneinit.bg4jts.cn", label: t("cl.home") },
          ].map((l) => (
            <a
              key={l.href}
              href={l.href}
              target="_blank"
              rel="noopener noreferrer"
              className="rounded-lg border border-emerald-600/20 bg-emerald-600/5 px-3 py-1.5 text-sm font-medium text-emerald-400 hover:border-emerald-600/40 transition-all"
            >
              {l.label}
            </a>
          ))}
        </div>

        {loading && (
          <div className="mt-12 flex items-center gap-2 font-mono text-sm text-zinc-500">
            <span className="h-2 w-2 animate-pulse rounded-full bg-emerald-500" />
            {t("cl.fetching")}
          </div>
        )}
        {!loading && error && (
          <div className="mt-12 rounded-xl glass p-6 font-mono text-xs text-zinc-500">{t("cl.err")}</div>
        )}

        <div className="mt-10 space-y-8">
          {releases.map((r) => (
            <article key={r.tag_name} className="rounded-2xl glass p-6">
              <div className="flex flex-wrap items-center gap-3">
                <a
                  href={r.html_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-mono text-lg font-bold text-white hover:text-emerald-400 transition-colors"
                >
                  {r.name || r.tag_name}
                </a>
                {r.prerelease && (
                  <span className="rounded-full border border-amber-500/20 bg-amber-500/5 px-2 py-0.5 font-mono text-[10px] text-amber-400">
                    {t("cl.prerelease")}
                  </span>
                )}
              </div>

              {/* 元数据 */}
              <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 font-mono text-xs text-zinc-600">
                <span>#{r.tag_name}</span>
                <span>{t("cl.published")}: {new Date(r.published_at || r.created_at).toLocaleDateString()}</span>
                <span>{t("cl.author")}: {t("cl.by")} {r.author?.login || "—"}</span>
              </div>

              {/* Release body 完整渲染 */}
              <div className="mt-4">{renderMd(r.body)}</div>

              {/* 资产下载 */}
              {r.assets && r.assets.length > 0 && (
                <div className="mt-4 border-t border-white/[0.04] pt-3">
                  <h3 className="mb-2 font-mono text-xs uppercase tracking-widest text-zinc-500">
                    📦 {t("cl.assets")}
                  </h3>
                  <div className="flex flex-wrap gap-2">
                    {r.assets.map((a) => (
                      <a
                        key={a.name}
                        href={a.browser_download_url}
                        className="rounded-lg border border-white/[0.06] px-3 py-1.5 font-mono text-xs text-zinc-400 hover:border-emerald-600/30 hover:text-emerald-400 transition-all"
                      >
                        {a.name} <span className="text-zinc-600">({fmtSize(a.size)})</span>
                      </a>
                    ))}
                  </div>
                </div>
              )}

              <div className="mt-4 border-t border-white/[0.04] pt-3">
                <code className="font-mono text-xs text-zinc-500">
                  <span className="text-emerald-500">$</span> npm i -g oneinit@{r.tag_name.replace(/^v/, "")}
                </code>
              </div>
            </article>
          ))}
          {!loading && !error && releases.length === 0 && (
            <div className="font-mono text-sm text-zinc-500">{t("cl.empty")}</div>
          )}
        </div>
      </div>
    </main>
  );
}
