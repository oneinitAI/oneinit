"use client";
import { useEffect, useState } from "react";
import { useLang } from "@/components/lang-provider";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";
import { Avatar } from "@/components/avatar";

type Contributor = {
  login: string;
  html_url: string;
  avatar_url: string;
  contributions: number;
  repos: string[];
  source: string[];
  tags: string[];
};

export default function ContributorsPage() {
  const { t } = useLang();
  const [contributors, setContributors] = useState<Contributor[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    fetch("/api/v1/contributors")
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((d: { contributors: Contributor[] }) => setContributors(d.contributors || []))
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  const total = contributors.reduce((s, c) => s + c.contributions, 0);
  const maxC = Math.max(...contributors.map((c) => c.contributions), 1);

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
      <Nav />
      <div className="mx-auto max-w-[1000px] px-6 pb-24 pt-28">
        <div className="mb-12 text-center">
          <span className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-4 py-1.5 font-mono text-xs tracking-widest text-emerald-500">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500" />
            {t("cb.badge")}
          </span>
          <h1 className="text-4xl font-bold text-white md:text-5xl">{t("cb.title")}</h1>
          <p className="mx-auto mt-4 max-w-[620px] text-zinc-400">{t("cb.desc")}</p>
          <div className="mt-6 flex items-center justify-center gap-3 text-sm">
            <span className="rounded-full border border-emerald-500/20 bg-emerald-500/5 px-3 py-1 font-mono text-emerald-400">
              {contributors.length} {t("cb.total")}
            </span>
            <span className="rounded-full border border-violet-500/20 bg-violet-500/5 px-3 py-1 font-mono text-violet-400">
              {total} {t("cb.contributions")}
            </span>
          </div>
        </div>

        {loading && (
          <p className="py-20 text-center font-mono text-sm text-zinc-500 animate-pulse">
            {t("rcp.loading")}
          </p>
        )}
        {error && (
          <p className="py-20 text-center font-mono text-sm text-red-400">{t("rcp.error")}</p>
        )}

        {!loading && !error && contributors.length === 0 && (
          <p className="py-20 text-center text-sm text-zinc-500">{t("cb.empty")}</p>
        )}

        {/* 排行榜：按贡献数排序的竖排列表，占满页面宽度 */}
        {contributors.length > 0 && (
          <>
            <div className="space-y-2.5">
              {contributors.map((c, i) => (
                <a
                  key={c.login}
                  href={c.html_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="glass glass-hover group flex items-center gap-3 rounded-2xl p-4 transition-all hover:-translate-y-0.5 sm:gap-5"
                >
                  <span className="w-8 shrink-0 text-center font-mono text-sm font-bold text-zinc-600 group-hover:text-emerald-400">
                    {i === 0 ? "🥇" : i === 1 ? "🥈" : i === 2 ? "🥉" : `#${i + 1}`}
                  </span>
                  <Avatar src={c.avatar_url} alt={c.login} size={44} />
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-1.5">
                      <span className="truncate text-sm font-semibold text-zinc-100 group-hover:text-emerald-300">
                        {c.login}
                      </span>
                      {c.tags?.map((tag) => (
                        <span
                          key={tag}
                          className="rounded bg-amber-500/10 px-1.5 py-0.5 font-mono text-[10px] text-amber-400"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                    <div className="mt-1.5 h-1.5 overflow-hidden rounded-full bg-zinc-800">
                      <div
                        className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-emerald-400 transition-all duration-700"
                        style={{ width: `${Math.max((c.contributions / maxC) * 100, 4)}%` }}
                      />
                    </div>
                    <div className="mt-1 font-mono text-[10px] text-zinc-500">
                      {c.repos.join(" / ") || c.source.join(" / ")}
                    </div>
                  </div>
                  <div className="shrink-0 text-right">
                    <div className="font-mono text-xl font-bold text-emerald-400">
                      {c.contributions}
                    </div>
                    <div className="text-[10px] text-zinc-500">{t("cb.contributions")}</div>
                  </div>
                </a>
              ))}
            </div>

            <div className="mt-10 rounded-2xl border border-emerald-500/15 bg-emerald-500/[0.04] p-8 text-center">
              <h2 className="text-2xl font-bold text-white">{t("cb.ctaTitle")}</h2>
              <p className="mx-auto mt-2 max-w-[560px] text-sm text-zinc-400">{t("cb.ctaDesc")}</p>
              <div className="mt-5 flex flex-wrap items-center justify-center gap-3">
                <a
                  href="https://github.com/oneinitAI/oneinit"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="rounded-lg border border-emerald-500/25 bg-emerald-500/10 px-4 py-2 text-sm font-semibold text-emerald-300 transition-all hover:border-emerald-400/50"
                >
                  oneinit · GitHub
                </a>
                <a
                  href="/recipes"
                  className="rounded-lg border border-white/[0.08] px-4 py-2 text-sm font-medium text-zinc-300 transition-all hover:border-emerald-600/30 hover:text-emerald-300"
                >
                  {t("nav.recipes")}
                </a>
              </div>
            </div>
          </>
        )}
      </div>
      <Footer />
    </main>
  );
}
