"use client";
import { useState } from "react";
import { useLang } from "./lang-provider";

export function Nav() {
  const { lang, toggle, t } = useLang();
  const [open, setOpen] = useState(false);
  const close = () => setOpen(false);

  const langButton = (
    <button
      onClick={toggle}
      className="rounded-lg border border-white/[0.06] px-2 py-1.5 font-mono text-xs font-medium text-zinc-400 hover:border-emerald-600/30 hover:text-emerald-400 transition-all"
      title={lang === "en" ? "切换中文" : "Switch to English"}
    >
      {lang === "en" ? "中文" : "EN"}
    </button>
  );

  const recipesLink = (
    <a
      href="/recipes"
      onClick={close}
      className="rounded-lg border border-white/[0.06] p-2 sm:px-3 sm:py-1.5 text-sm font-medium text-zinc-400 hover:border-emerald-600/30 hover:text-emerald-400 transition-all flex items-center gap-1.5"
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="shrink-0"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2m-6 9 2 2 4-4"/></svg>
      <span className="hidden sm:inline">{t("nav.recipes")}</span>
    </a>
  );

  const contributorsLink = (
    <a
      href="/contributors"
      onClick={close}
      className="rounded-lg border border-white/[0.06] p-2 sm:px-3 sm:py-1.5 text-sm font-medium text-zinc-400 hover:border-amber-500/30 hover:text-amber-300 transition-all flex items-center gap-1.5"
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="shrink-0"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zm14 10v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/></svg>
      <span className="hidden sm:inline">{t("nav.contributors")}</span>
    </a>
  );

  const afdianLink = (
    <a
      href="https://ifdian.net/a/BG4JTS" target="_blank" rel="noopener noreferrer"
      className="rounded-lg border border-violet-500/25 bg-gradient-to-r from-violet-600/15 to-fuchsia-600/10 px-2 py-1.5 sm:px-3 sm:py-1.5 text-sm font-bold text-violet-300 hover:border-violet-400/50 hover:text-violet-200 transition-all flex items-center gap-1.5"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" className="shrink-0"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/></svg>
      <span className="sm:hidden text-xs">爱发电</span>
      <span className="hidden sm:inline">{t("afdian.cta")}</span>
    </a>
  );

  const sponsorLink = (
    <a
      href="https://github.com/sponsors/BG4JTS" target="_blank" rel="noopener noreferrer"
      className="rounded-lg border border-pink-500/20 bg-pink-500/5 p-2 sm:px-3 sm:py-1.5 text-sm font-medium text-pink-400 hover:border-pink-500/40 hover:text-pink-300 transition-all flex items-center gap-1.5"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" className="shrink-0"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/></svg>
      <span className="hidden sm:inline">{t("nav.sponsor")}</span>
    </a>
  );

  const npmLink = (
    <a
      href="https://www.npmjs.com/package/oneinit" target="_blank" rel="noopener noreferrer"
      className="rounded-lg border border-white/[0.06] p-2 sm:px-3 sm:py-1.5 text-sm font-medium text-zinc-400 hover:border-red-500/30 hover:text-red-400 transition-all flex items-center gap-1.5"
    >
      <span className="flex h-4 w-4 shrink-0 items-center justify-center rounded-sm bg-red-500 font-mono text-[9px] font-bold text-white leading-none">n</span>
      <span className="hidden sm:inline">{t("nav.npm")}</span>
    </a>
  );

  const githubLink = (
    <a
      href="https://github.com/oneinitAI/oneinit" target="_blank" rel="noopener noreferrer"
      className="rounded-lg border border-white/[0.06] p-2 sm:px-3 sm:py-1.5 text-sm font-medium text-zinc-400 hover:border-emerald-600/30 hover:text-emerald-400 transition-all flex items-center gap-1.5"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" className="shrink-0"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
      <span className="hidden sm:inline">{t("nav.github")}</span>
    </a>
  );

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 border-b border-white/[0.04] bg-[#0a0a0f]/80 backdrop-blur-xl">
      <div className="mx-auto flex max-w-[1200px] items-center justify-between px-6 h-16">
        <a href="#" className="flex items-center gap-2.5 font-sans font-bold text-lg tracking-tight text-white">
          <img src="/logo.png" alt="OneInit" className="h-8 w-auto brightness-110 drop-shadow-[0_0_8px_rgba(5,150,105,0.3)]" />
          oneinit
        </a>

        {/* 桌面端：横排（md 以上） */}
        <div className="hidden md:flex items-center gap-1.5 sm:gap-3">
          {recipesLink}
          {contributorsLink}
          {langButton}
          {afdianLink}
          {sponsorLink}
          {npmLink}
          {githubLink}
        </div>

        {/* 移动端：汉堡按钮（竖排菜单） */}
        <div className="md:hidden flex items-center gap-1.5">
          {langButton}
          <button
            onClick={() => setOpen(!open)}
            aria-label="menu"
            aria-expanded={open}
            className="rounded-lg border border-white/[0.06] p-2 text-zinc-400 hover:border-emerald-600/30 hover:text-emerald-400 transition-all"
          >
            {open ? (
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
            ) : (
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M3 6h18M3 12h18M3 18h18"/></svg>
            )}
          </button>
        </div>
      </div>

      {/* 移动端：竖排下拉菜单 */}
      {open && (
        <div className="md:hidden border-t border-white/[0.04] bg-[#0a0a0f]/95 backdrop-blur-xl px-4 py-3 flex flex-col gap-2">
          {recipesLink}
          {contributorsLink}
          {afdianLink}
          {sponsorLink}
          {npmLink}
          {githubLink}
          <div className="mt-1 flex items-center justify-between border-t border-white/[0.04] pt-2">
            <span className="text-xs text-zinc-500">{lang === "en" ? "Language" : "语言"}</span>
            {langButton}
          </div>
        </div>
      )}
    </nav>
  );
}
