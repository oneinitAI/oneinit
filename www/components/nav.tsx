"use client";

export function Nav() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 border-b border-emerald-500/10 bg-zinc-950/90 backdrop-blur-xl">
      <div className="mx-auto flex max-w-[1200px] items-center justify-between px-6 h-16">
        <a href="#" className="flex items-center gap-2">
          <span className="flex h-7 w-7 items-center justify-center rounded-md bg-neon font-mono text-xs font-bold text-zinc-950 shadow-[0_0_12px_rgba(0,255,136,0.3)]">
            {"</>"}
          </span>
          <span className="font-mono text-sm font-bold tracking-tight">oneinit</span>
        </a>
        <div className="flex items-center gap-6">
          <a href="#commands" className="hidden text-sm text-zinc-500 transition-colors hover:text-neon sm:block">Commands</a>
          <a href="#install" className="hidden text-sm text-zinc-500 transition-colors hover:text-neon sm:block">Install</a>
          <a
            href="https://github.com/BG4JTS/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            data-clickable
            className="rounded-lg border border-zinc-700 px-4 py-1.5 text-sm font-medium text-zinc-300 transition-all hover:border-emerald-500/50 hover:text-neon active:translate-y-px"
          >
            GitHub
          </a>
        </div>
      </div>
    </nav>
  );
}
