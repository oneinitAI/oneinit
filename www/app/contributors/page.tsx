"use client";
import { useCallback, useEffect, useState } from "react";
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

  const loadContributors = useCallback(() => {
    fetch("/api/v1/contributors")
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((d: { contributors: Contributor[] }) => setContributors(d.contributors || []))
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    loadContributors();
  }, [loadContributors]);

  const top5 = contributors.slice(0, 5);
  const rest = contributors.slice(5);
  const total = contributors.reduce((s, c) => s + c.contributions, 0);
  const maxC = Math.max(...contributors.map((c) => c.contributions), 1);

  const Tag = ({ tag }: { tag: string }) => (
    <span className="rounded bg-amber-500/10 px-1.5 py-0.5 font-mono text-[10px] text-amber-400">
      {tag}
    </span>
  );

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
      <Nav />
      <div className="mx-auto max-w-[1000px] px-6 pb-24 pt-28">
        <div className="mb-10 text-center">
          <span className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-4 py-1.5 font-mono text-xs tracking-widest text-emerald-500">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500" />
            {t("cb.badge")}
          </span>
          <h1 className="text-4xl font-bold text-white md:text-5xl">{t("cb.title")}</h1>
          <p className="mx-auto mt-4 max-w-[620px] text-zinc-400">{t("cb.desc")}</p>
          <div className="mt-5 flex flex-wrap items-center justify-center gap-3 text-sm">
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

        {contributors.length > 0 && (
          <>
            {/* TOP 5 排行榜 */}
            <h2 className="mb-4 font-mono text-sm font-bold tracking-widest text-zinc-300">
              {t("cb.top5")}
            </h2>
            <div className="space-y-2.5">
              {top5.map((c, i) => (
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
                        <Tag key={tag} tag={tag} />
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

            {/* 其余贡献者：头像并排展示 */}
            {rest.length > 0 && (
              <>
                <h2 className="mb-4 mt-10 font-mono text-sm font-bold tracking-widest text-zinc-300">
                  {t("cb.others")} ({rest.length})
                </h2>
                <div className="flex flex-wrap gap-3">
                  {rest.map((c) => (
                    <a
                      key={c.login}
                      href={c.html_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="glass group flex items-center gap-2.5 rounded-xl px-3 py-2 transition-all hover:-translate-y-0.5"
                    >
                      <Avatar src={c.avatar_url} alt={c.login} size={36} />
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-1">
                          <span className="text-sm font-semibold text-zinc-200 group-hover:text-emerald-300">
                            {c.login}
                          </span>
                        </div>
                        <div className="flex flex-wrap gap-1">
                          {c.tags?.map((tag) => (
                            <Tag key={tag} tag={tag} />
                          ))}
                          {(!c.tags || c.tags.length === 0) && (
                            <span className="font-mono text-[10px] text-zinc-500">
                              {c.contributions}
                            </span>
                          )}
                        </div>
                      </div>
                    </a>
                  ))}
                </div>
              </>
            )}

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
