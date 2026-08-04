"use client";
import { useEffect, useState } from "react";
import { useLang } from "@/components/lang-provider";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";

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

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
      <Nav />
      <div className="mx-auto max-w-[1100px] px-6 pb-24 pt-28">
        <div className="mb-12 text-center">
          <span className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-4 py-1.5 font-mono text-xs tracking-widest text-emerald-500">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500" />
            {t("cb.badge")}
          </span>
          <h1 className="text-4xl font-bold text-white md:text-5xl">{t("cb.title")}</h1>
          <p className="mx-auto mt-4 max-w-[620px] text-zinc-400">{t("cb.desc")}</p>
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
            <div className="mb-10 grid grid-cols-2 gap-4 lg:grid-cols-4">
              <div className="glass rounded-2xl p-5 text-center">
                <div className="text-3xl font-bold text-emerald-400">{contributors.length}</div>
                <div className="mt-1 text-xs text-zinc-500">{t("cb.total")}</div>
              </div>
              <div className="glass rounded-2xl p-5 text-center">
                <div className="text-3xl font-bold text-violet-400">{total}</div>
                <div className="mt-1 text-xs text-zinc-500">{t("cb.contributions")}</div>
              </div>
              <div className="glass col-span-2 hidden rounded-2xl p-5 md:block" />
            </div>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {contributors.map((c, i) => (
                <a
                  key={c.login}
                  href={c.html_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  data-aos="fade-up"
                  data-aos-delay={(i % 3) * 80}
                  className="glass glass-hover group flex items-center gap-3 rounded-2xl p-4 transition-all hover:-translate-y-0.5"
                >
                  <img
                    src={c.avatar_url}
                    alt={c.login}
                    loading="lazy"
                    className="h-12 w-12 shrink-0 rounded-full border border-white/10"
                  />
                  <div className="min-w-0">
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
                    <div className="mt-0.5 font-mono text-[11px] text-zinc-500">
                      {c.contributions} · {c.repos.join(" / ") || c.source.join(" / ")}
                    </div>
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
