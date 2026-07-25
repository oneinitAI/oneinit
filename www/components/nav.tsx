"use client";
export function Nav() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 border-b border-[rgba(255,255,255,0.04)] bg-[rgba(5,5,16,0.8)] backdrop-blur-xl">
      <div className="mx-auto flex max-w-[1200px] items-center justify-between px-6 h-16">
        <a href="#" className="flex items-center gap-2 font-mono font-bold text-lg tracking-tight">
          <span className="text-gradient text-xl">{">_"}</span>
          oneinit
        </a>
        <a href="https://github.com/BG4JTS/oneinit" target="_blank" rel="noopener noreferrer"
            className="rounded-lg border border-[rgba(255,255,255,0.08)] px-4 py-1.5 text-sm font-medium text-zinc-400 glass-hover transition-all">
          GitHub
        </a>
      </div>
    </nav>
  );
}
