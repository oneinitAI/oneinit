"use client";
import { useEffect, useState } from "react";
import { useLang } from "@/components/lang-provider";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";

type Stats = {
  ok?: boolean;
  total_recipes: number;
  pending_uploads: number;
  last_updated: string | null;
  top_tags: { tag: string; count: number }[];
  top_maintainers: { name: string; count: number }[];
  platform_coverage: { windows: number; linux: number; darwin: number };
  pending_uploads_list: { number: number; title: string; url: string; author: string; created_at: string }[];
  recipes: { name: string; description: string; latest: string; tags: string[]; maintainers: string[] }[];
};

type Contributor = {
  login: string;
  html_url: string;
  avatar_url: string;
  contributions: number;
  repos: string[];
  source: string[];
};

export default function RecipesPage() {
  const { t } = useLang();
  const [data, setData] = useState<Stats | null>(null);
  const [contributors, setContributors] = useState<Contributor[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    fetch("/api/v1/stats")
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((d: Stats) => setData(d))
      .catch(() => setError(true))
      .finally(() => setLoading(false));
    fetch("/api/v1/contributors")
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((d: { contributors: Contributor[] }) => setContributors(d.contributors || []))
      .catch(() => {});
  }, []);

  const pc = data?.platform_coverage ?? { windows: 0, linux: 0, darwin: 0 };
  const maxPlat = Math.max(pc.windows, pc.linux, pc.darwin, 1);
  const maxTag = Math.max(...(data?.top_tags ?? []).map((x) => x.count), 1);

  const cards = [
    { label: t("rcp.total"), value: data?.total_recipes ?? "–", accent: "text-emerald-400" },
    { label: t("rcp.pending"), value: data?.pending_uploads ?? "–", accent: "text-amber-400" },
    { label: t("rcp.tags.count"), value: data?.top_tags.length ?? "–", accent: "text-violet-400" },
    { label: t("rcp.maintainers.count"), value: data?.top_maintainers.length ?? "–", accent: "text-sky-400" },
  ];

  const platforms = [
    { key: "windows", label: "Windows", value: pc.windows },
    { key: "linux", label: "Linux", value: pc.linux },
    { key: "darwin", label: "macOS", value: pc.darwin },
  ];

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
      <Nav />
      <div className="mx-auto max-w-[1100px] px-6 pb-24 pt-28">
        <div className="mb-12 text-center">
          <span className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-4 py-1.5 font-mono text-xs tracking-widest text-emerald-500">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500" />
            {t("rcp.badge")}
          </span>
          <h1 className="text-4xl font-bold text-white md:text-5xl">{t("rcp.title")}</h1>
          <p className="mx-auto mt-4 max-w-[620px] text-zinc-400">{t("rcp.desc")}</p>
          {data?.last_updated && (
            <p className="mt-3 font-mono text-xs text-zinc-600">
              INDEX {data.last_updated}
            </p>
          )}
        </div>

        {loading && (
          <p className="py-20 text-center font-mono text-sm text-zinc-500 animate-pulse">
            {t("rcp.loading")}
          </p>
        )}
        {error && (
          <p className="py-20 text-center font-mono text-sm text-red-400">{t("rcp.error")}</p>
        )}

        {data && (
          <>
            {/* 统计卡片 */}
            <div className="mb-10 grid grid-cols-2 gap-4 lg:grid-cols-4">
              {cards.map((c, i) => (
                <div key={i} className="glass rounded-2xl p-5 text-center">
                  <div className={`text-3xl font-bold ${c.accent}`}>{c.value}</div>
                  <div className="mt-1 text-xs text-zinc-500">{c.label}</div>
                </div>
              ))}
            </div>

            <div className="mb-10 grid grid-cols-1 gap-4 lg:grid-cols-2">
              {/* 平台覆盖 */}
              <div className="glass rounded-2xl p-6">
                <h2 className="mb-4 font-mono text-sm font-bold text-white">{t("rcp.platform")}</h2>
                {platforms.map((p) => (
                  <div key={p.key} className="mb-3">
                    <div className="mb-1 flex items-center justify-between text-xs">
                      <span className="text-zinc-400">{p.label}</span>
                      <span className="font-mono text-zinc-300">
                        {p.value}/{data.total_recipes}
                      </span>
                    </div>
                    <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                      <div
                        className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-emerald-400 transition-all duration-700"
                        style={{ width: `${(p.value / maxPlat) * 100}%` }}
                      />
                    </div>
                  </div>
                ))}
              </div>

              {/* 热门标签 */}
              <div className="glass rounded-2xl p-6">
                <h2 className="mb-4 font-mono text-sm font-bold text-white">{t("rcp.tags")}</h2>
                {(data.top_tags.length ? data.top_tags : [{ tag: "runtime", count: 0 }]).map((x) => (
                  <div key={x.tag} className="mb-3">
                    <div className="mb-1 flex items-center justify-between text-xs">
                      <span className="font-mono text-emerald-300">#{x.tag}</span>
                      <span className="font-mono text-zinc-400">{x.count}</span>
                    </div>
                    <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                      <div
                        className="h-full rounded-full bg-gradient-to-r from-violet-500 to-fuchsia-400 transition-all duration-700"
                        style={{ width: `${(x.count / maxTag) * 100}%` }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* 待审核上传 */}
            <div className="glass mb-10 rounded-2xl p-6">
              <h2 className="mb-4 font-mono text-sm font-bold text-white">
                {t("rcp.pendingTitle")}
              </h2>
              {data.pending_uploads_list.length === 0 ? (
                <p className="text-sm text-zinc-500">{t("rcp.noPending")}</p>
              ) : (
                <ul className="space-y-2">
                  {data.pending_uploads_list.map((p) => (
                    <li key={p.number}>
                      <a
                        href={p.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="group flex items-center justify-between rounded-lg border border-white/[0.05] px-3 py-2 text-sm hover:border-emerald-600/30 hover:bg-emerald-500/5"
                      >
                        <span className="truncate text-zinc-300 group-hover:text-emerald-300">
                          #{p.number} · {p.title}
                        </span>
                        <span className="ml-3 shrink-0 font-mono text-xs text-zinc-500">
                          {p.author}
                        </span>
                      </a>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            {/* 配方表格 */}
            <div className="glass overflow-hidden rounded-2xl">
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead>
                    <tr className="border-b border-white/[0.06] font-mono text-xs text-zinc-500">
                      <th className="px-5 py-3">{t("rcp.thName")}</th>
                      <th className="px-5 py-3">{t("rcp.thDesc")}</th>
                      <th className="px-5 py-3">{t("rcp.thLatest")}</th>
                      <th className="hidden px-5 py-3 md:table-cell">{t("rcp.thTags")}</th>
                      <th className="hidden px-5 py-3 lg:table-cell">{t("rcp.thMaintainers")}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.recipes.map((r) => (
                      <tr
                        key={r.name}
                        className="border-b border-white/[0.04] transition-colors hover:bg-white/[0.02]"
                      >
                        <td className="px-5 py-3 font-mono text-emerald-300">{r.name}</td>
                        <td className="max-w-[300px] px-5 py-3 text-zinc-400">{r.description}</td>
                        <td className="px-5 py-3 font-mono text-zinc-300">{r.latest}</td>
                        <td className="hidden px-5 py-3 md:table-cell">
                          <div className="flex flex-wrap gap-1">
                            {r.tags.slice(0, 3).map((tag) => (
                              <span
                                key={tag}
                                className="rounded bg-zinc-800/80 px-1.5 py-0.5 font-mono text-[10px] text-violet-300"
                              >
                                {tag}
                              </span>
                            ))}
                          </div>
                        </td>
                        <td className="hidden px-5 py-3 lg:table-cell">
                          <span className="font-mono text-xs text-zinc-500">
                            {r.maintainers.join(", ") || "–"}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            {/* 贡献者名单（与 GitHub 同步） */}
            <div className="glass mt-10 rounded-2xl p-6">
              <h2 className="mb-1 font-mono text-sm font-bold text-white">
                {t("rcp.contributors")}
              </h2>
              <p className="mb-5 text-xs text-zinc-500">{t("rcp.contributorsDesc")}</p>
              {contributors.length === 0 ? (
                <p className="text-sm text-zinc-500">{t("rcp.contributorsEmpty")}</p>
              ) : (
                <div className="flex flex-wrap gap-3">
                  {contributors.map((c) => (
                    <a
                      key={c.login}
                      href={c.html_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="group flex items-center gap-2.5 rounded-xl border border-white/[0.05] px-3 py-2 transition-all hover:border-emerald-600/30 hover:bg-emerald-500/5"
                    >
                      <img
                        src={c.avatar_url}
                        alt={c.login}
                        className="h-8 w-8 rounded-full border border-white/10"
                        loading="lazy"
                      />
                      <div className="leading-tight">
                        <div className="text-sm font-semibold text-zinc-200 group-hover:text-emerald-300">
                          {c.login}
                        </div>
                        <div className="font-mono text-[10px] text-zinc-500">
                          {c.contributions} · {c.repos.join(" / ")}
                        </div>
                      </div>
                    </a>
                  ))}
                </div>
              )}
            </div>

            {/* 贡献 CTA */}
            <div className="mt-10 rounded-2xl border border-emerald-500/15 bg-emerald-500/[0.04] p-8 text-center">
              <h2 className="text-2xl font-bold text-white">{t("rcp.contributeTitle")}</h2>
              <p className="mx-auto mt-2 max-w-[560px] text-sm text-zinc-400">
                {t("rcp.contributeDesc")}
              </p>
              <div className="mx-auto mt-6 max-w-[520px] space-y-2 text-left font-mono text-xs text-zinc-300">
                <div className="rounded-lg border border-white/[0.06] bg-black/30 px-4 py-3">
                  <span className="text-emerald-400">$</span> oneinit recipe wizard my-tool
                  <span className="text-zinc-600">  # 无现成配方时</span>
                </div>
                <div className="rounded-lg border border-white/[0.06] bg-black/30 px-4 py-3">
                  <span className="text-emerald-400">$</span> oneinit recipe contribute my-tool.yaml
                  <span className="text-zinc-600">  # 上传到本平台</span>
                </div>
              </div>
            </div>
          </>
        )}
      </div>
      <Footer />
    </main>
  );
}
