"use client";
import { useLang } from "./lang-provider";

export function Footer() {
  const { t } = useLang();
  return (
    <footer className="border-t border-white/[0.04] py-20 text-center">
      <div className="mx-auto max-w-[600px] px-6">
        <img src="/logo.png" alt="OneInit" className="mx-auto h-12 w-auto mb-6" />
        <h2 className="text-3xl font-bold tracking-tight text-white md:text-5xl">
          {t("ft.title1")} <span className="text-emerald-500">{t("ft.title2")}</span>.
        </h2>
        <p className="mt-4 text-zinc-500">{t("ft.stats")}</p>
        <div className="mt-8 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a href="#install" className="rounded-xl bg-emerald-600 px-8 py-3.5 font-bold text-white transition-all hover:bg-emerald-500 hover:shadow-lg hover:shadow-emerald-600/20 active:scale-[0.98]">
            {t("ft.cta")}
          </a>
          <a href="https://github.com/oneinitAI/oneinit" target="_blank" rel="noopener noreferrer" className="glass glass-hover rounded-xl px-8 py-3.5 font-bold text-zinc-300 transition-all">
            {t("ft.github")}
          </a>
        </div>
        <div className="mt-12 flex items-center justify-center gap-6 text-sm text-zinc-600">
          <a href="https://github.com/oneinitAI/oneinit" className="hover:text-zinc-400">GitHub</a>
          <a href="https://www.npmjs.com/package/oneinit" className="hover:text-zinc-400">npm</a>
          <a href="/changelog" className="hover:text-zinc-400">{t("cl.title")}</a>
          <a href="/terms" className="hover:text-zinc-400">{t("ft.terms")}</a>
          <span>GPL-3.0</span>
        </div>
        <p className="mt-4 font-mono text-xs text-zinc-700">{t("ft.built")}</p>
        <div className="mt-8 flex flex-col items-center gap-3">
          <p className="text-sm text-zinc-500">{t("ft.support")}</p>
          <div className="flex items-center gap-3">
            <a href="https://github.com/sponsors/BG4JTS" target="_blank" rel="noopener noreferrer"
               className="rounded-lg border border-pink-500/20 bg-pink-500/5 px-4 py-2 text-sm font-medium text-pink-400 hover:border-pink-500/40 hover:text-pink-300 transition-all flex items-center gap-1.5">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/></svg>
              GitHub Sponsors
            </a>
            <a href="https://opencollective.com/bg4jts" target="_blank" rel="noopener noreferrer"
               className="rounded-lg border border-white/[0.06] px-4 py-2 text-sm font-medium text-zinc-400 hover:border-blue-500/30 hover:text-blue-400 transition-all">
              Open Collective
            </a>
          </div>
        </div>
        <p className="mt-6 font-mono text-xs text-zinc-800">
          &copy; {new Date().getFullYear()} BG4JTS. All rights reserved.
        </p>
      </div>
    </footer>
  );
}
