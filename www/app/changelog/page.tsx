"use client";
import { useEffect, useState } from "react";
import { useLang } from "@/components/lang-provider";

const REPO = "oneinitAI/oneinit";
const API = `https://api.github.com/repos/${REPO}/releases`;

type Release = {
  tag_name: string;
  name: string;
  published_at: string;
  prerelease: boolean;
  body: string;
  html_url: string;
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

  // 解析 release body：拆出 Security / Changelog 段落
  const extractSection = (body: string, header: string) => {
    const lines = body.split("\n");
    const start = lines.findIndex((l) => l.toLowerCase().includes(header.toLowerCase()));
    if (start < 0) return "";
    const out: string[] = [];
    for (const l of lines.slice(start + 1)) {
      if (l.startsWith("##") || l.startsWith("### 📦") || l.startsWith("### 🔗")) break;
      if (l.startsWith("###") && !out.length) continue;
      out.push(l);
    }
    return out.filter((l) => l.trim()).join("\n").trim();
  };

  const renderMdList = (text: string) => {
    if (!text) return null;
    return text.split("\n").map((line, i) => (
      <li key={i} className="text-sm leading-relaxed text-zinc-400">
        {line.replace(/^- /, "")}
      </li>
    ));
  };

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
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
          {releases.map((r) => {
            const security = extractSection(r.body, "security fix");
            const changelog = extractSection(r.body, "changelog");
            return (
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
                  <span className="ml-auto font-mono text-xs text-zinc-600">
                    {new Date(r.published_at).toLocaleDateString()}
                  </span>
                </div>

                {security && (
                  <div className="mt-4 rounded-xl border border-rose-500/10 bg-rose-500/[0.02] p-4">
                    <h3 className="mb-2 font-mono text-xs uppercase tracking-widest text-rose-400">
                      🔒 {t("cl.security")}
                    </h3>
                    <ul className="list-disc space-y-1 pl-4">{renderMdList(security)}</ul>
                  </div>
                )}

                {changelog && (
                  <div className="mt-4">
                    <h3 className="mb-2 font-mono text-xs uppercase tracking-widest text-emerald-500">
                      📝 {t("cl.changelog")}
                    </h3>
                    <ul className="list-disc space-y-1 pl-4">{renderMdList(changelog)}</ul>
                  </div>
                )}

                <div className="mt-4 border-t border-white/[0.04] pt-3">
                  <code className="font-mono text-xs text-zinc-500">
                    <span className="text-emerald-500">$</span> npm i -g oneinit@{r.tag_name.replace(/^v/, "")}
                  </code>
                </div>
              </article>
            );
          })}
          {!loading && !error && releases.length === 0 && (
            <div className="font-mono text-sm text-zinc-500">{t("cl.empty")}</div>
          )}
        </div>
      </div>
    </main>
  );
}
