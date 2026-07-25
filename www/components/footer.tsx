"use client";

export function Footer() {
  return (
    <footer className="relative border-t border-zinc-800 py-24 md:py-32 overflow-hidden">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_center,_rgba(16,185,129,0.1)_0%,_transparent_70%)]" />
      <div className="relative z-10 mx-auto max-w-[800px] px-6 text-center">
        <h2 className="mb-8 text-4xl font-black tracking-tight md:text-6xl">
          Ready to{" "}
          <span className="chromatic bg-gradient-to-r from-neon to-emerald-400 bg-clip-text text-transparent">
            ship faster
          </span>
          ?
        </h2>

        <p className="mx-auto mb-10 max-w-[500px] text-lg text-zinc-500">
          One command. Every tool. Complete environment. Zero sudo.
        </p>

        <div className="flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a
            href="#install"
            data-clickable
            className="group relative overflow-hidden rounded-lg bg-emerald-500 px-10 py-4 font-bold text-zinc-950 transition-all hover:bg-neon hover:shadow-[0_0_50px_rgba(0,255,136,0.3)] active:scale-[0.97]"
          >
            <span className="relative z-10">Get Started</span>
            <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/20 to-transparent group-hover:translate-x-full transition-transform duration-700" />
          </a>
          <a
            href="https://github.com/BG4JTS/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            data-clickable
            className="rounded-lg border border-zinc-700 px-10 py-4 font-bold text-zinc-300 transition-all hover:border-emerald-500/50 hover:text-emerald-400 active:scale-[0.97]"
          >
            View on GitHub
          </a>
        </div>

        <div className="mt-16 flex items-center justify-center gap-8 text-sm text-zinc-600">
          <a href="https://github.com/BG4JTS/oneinit" target="_blank" rel="noopener noreferrer" className="transition-colors hover:text-zinc-400">GitHub</a>
          <a href="https://www.npmjs.com/package/oneinit" target="_blank" rel="noopener noreferrer" className="transition-colors hover:text-zinc-400">npm</a>
          <span>GPL-3.0</span>
          <a href="/README_CN.md" className="transition-colors hover:text-zinc-400">中文</a>
        </div>
        <p className="mt-6 font-mono text-xs text-zinc-700">
          Built with Rust · Zero runtime · 26 tests · No bullshit
        </p>
      </div>
    </footer>
  );
}
