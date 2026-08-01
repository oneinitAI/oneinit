"use client";
import { useEffect, useState } from "react";
import { useLang } from "./lang-provider";

// 内置配方（无需联网，始终显示）
const BUILTIN = [
  { name: "python3.11", version: "3.11.9", description: "Python + pip + 清华源", tags: ["runtime"], source: "builtin" },
  { name: "node20", version: "20.18.1", description: "Node.js 20 LTS + npm 淘宝源", tags: ["runtime", "javascript", "typescript"], source: "builtin" },
  { name: "go", version: "1.23.4", description: "Go 工具链", tags: ["runtime"], source: "builtin" },
  { name: "java17", version: "17.0.20+8", description: "Temurin JDK 17 LTS", tags: ["runtime", "java"], source: "builtin" },
];

const REGISTRY_URL =
  "https://raw.githubusercontent.com/oneinitAI/oneinit-recipes/main/INDEX.json";

type RecipeItem = {
  name: string;
  version: string;
  description: string;
  tags: string[];
  source: string;
};

export function Recipes() {
  const { t } = useLang();
  const [remote, setRemote] = useState<RecipeItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    fetch(REGISTRY_URL)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error("fetch failed"))))
      .then((data) => {
        const pkgs: RecipeItem[] = Object.entries(data.packages || {}).map(
          ([name, e]: [string, any]) => ({
            name,
            version: e.latest,
            description: e.description || "",
            tags: e.tags || [],
            source: "remote",
          })
        );
        setRemote(pkgs);
      })
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  // 合并内置 + 远程，内置优先（同名去重）
  const all: RecipeItem[] = [...BUILTIN];
  const seen = new Set(all.map((r) => r.name));
  for (const r of remote) {
    if (!seen.has(r.name)) all.push(r);
  }

  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <div className="mb-14 text-center" data-aos="fade-up">
          <span className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-4 py-1.5 font-mono text-xs tracking-widest text-emerald-500">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500" />
            {t("rc.badge")}
          </span>
          <h2 className="text-3xl font-bold text-white md:text-5xl">
            {t("rc.title1")}<br />
            <span className="text-zinc-600">{t("rc.title2", { n: all.length })}</span>
          </h2>
          <p className="mx-auto mt-4 max-w-[560px] text-zinc-400">{t("rc.desc")}</p>
        </div>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {all.map((r, i) => (
            <div
              key={r.name}
              data-aos="fade-up"
              data-aos-delay={(i % 3) * 100}
              className="glass glass-hover group flex flex-col rounded-2xl p-5 transition-all hover:-translate-y-1"
            >
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2">
                  <span
                    className={`rounded px-1.5 py-0.5 font-mono text-[10px] font-bold ${
                      r.source === "builtin"
                        ? "bg-emerald-500/15 text-emerald-400"
                        : "bg-cyan-500/15 text-cyan-400"
                    }`}
                  >
                    {r.source === "builtin" ? t("rc.builtin") : t("rc.remote")}
                  </span>
                  <span className="font-mono text-sm font-bold text-white">{r.name}</span>
                </div>
                <span className="font-mono text-xs text-zinc-500">v{r.version}</span>
              </div>

              <p className="mt-2 min-h-[2.5rem] text-sm leading-snug text-zinc-400">
                {r.description}
              </p>

              <div className="mt-3 flex flex-wrap gap-1.5">
                {r.tags.slice(0, 3).map((t) => (
                  <span
                    key={t}
                    className="rounded-full border border-white/[0.06] px-2 py-0.5 font-mono text-[10px] text-zinc-500"
                  >
                    {t}
                  </span>
                ))}
              </div>

              <div className="mt-4 flex items-center justify-between border-t border-white/[0.04] pt-3">
                <code className="truncate font-mono text-xs text-zinc-500">
                  <span className="text-emerald-500">$</span> oneinit install {r.name}
                </code>
              </div>
            </div>
          ))}

          {/* 加载/错误状态卡片 */}
          {loading && (
            <div className="glass flex items-center justify-center rounded-2xl p-6 font-mono text-sm text-zinc-500">
              <span className="mr-2 h-2 w-2 animate-pulse rounded-full bg-emerald-500" />
              {t("rc.fetching")}
            </div>
          )}
          {!loading && error && (
            <div className="glass flex items-center justify-center rounded-2xl p-6 font-mono text-xs text-zinc-600">
              {t("rc.err")}
            </div>
          )}
        </div>

        <p className="mt-8 text-center font-mono text-xs text-zinc-600">
          {t("rc.cta")}
          <a
            href="https://github.com/oneinitAI/oneinit-recipes/issues/new?assignees=&labels=recipe&projects=&template=recipe_request.yml"
            target="_blank"
            rel="noopener noreferrer"
            className="ml-1 text-emerald-500 hover:text-emerald-400"
          >
            github.com/oneinitAI/oneinit-recipes
          </a>
        </p>
      </div>
    </section>
  );
}
