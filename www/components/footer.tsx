"use client";
export function Footer() {
  return (
    <footer className="border-t border-[rgba(255,255,255,0.04)] py-20 text-center">
      <div className="mx-auto max-w-[600px] px-6">
        <h2 className="text-3xl font-bold tracking-tight md:text-5xl">
          Ready to <span className="text-gradient">ship faster</span>?
        </h2>
        <p className="mt-4 text-zinc-500">One command. Every tool. Complete environment. Zero sudo.</p>
        <div className="mt-8 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a href="#install" className="rounded-xl bg-gradient-to-r from-cyan to-purple px-8 py-3.5 font-bold text-white shadow-lg shadow-purple/25 transition-all hover:scale-[1.02] active:scale-[0.98]">
            Get Started
          </a>
          <a href="https://github.com/BG4JTS/oneinit" target="_blank" rel="noopener noreferrer" className="glass glass-hover rounded-xl px-8 py-3.5 font-bold text-zinc-300 transition-all">
            View on GitHub
          </a>
        </div>
        <div className="mt-12 flex items-center justify-center gap-6 text-sm text-zinc-600">
          <a href="https://github.com/BG4JTS/oneinit" className="hover:text-zinc-400">GitHub</a>
          <a href="https://www.npmjs.com/package/oneinit" className="hover:text-zinc-400">npm</a>
          <span>GPL-3.0</span>
        </div>
        <p className="mt-4 font-mono text-xs text-zinc-700">Built with Rust · Zero runtime · 26 tests</p>
      </div>
    </footer>
  );
}
