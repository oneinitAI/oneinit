"use client"; export function Nav() { return (
<nav className="fixed top-0 left-0 right-0 z-50 border-b border-white/[0.04] bg-[#0a0a0f]/80 backdrop-blur-xl">
  <div className="mx-auto flex max-w-[1200px] items-center justify-between px-6 h-16">
    <a href="#" className="flex items-center gap-2.5 font-sans font-bold text-lg tracking-tight text-white">
      <img src="/logo.png" alt="OneInit" className="h-8 w-auto brightness-110 drop-shadow-[0_0_8px_rgba(5,150,105,0.3)]" />
      oneinit
    </a>
    <div className="flex items-center gap-3">
    <a href="https://www.npmjs.com/package/oneinit" target="_blank" rel="noopener noreferrer"
       className="rounded-lg border border-white/[0.06] px-3 py-1.5 text-sm font-medium text-zinc-400 hover:border-red-500/30 hover:text-red-400 transition-all flex items-center gap-1.5">
      <span className="flex h-4 w-4 items-center justify-center rounded-sm bg-red-500 font-mono text-[9px] font-bold text-white leading-none">n</span>
      npm
    </a>
    <a href="https://github.com/BG4JTS/oneinit" target="_blank" rel="noopener noreferrer"
       className="rounded-lg border border-white/[0.06] px-3 py-1.5 text-sm font-medium text-zinc-400 hover:border-emerald-600/30 hover:text-emerald-400 transition-all flex items-center gap-1.5">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
      GitHub
    </a>
    </div>
  </div>
</nav>); }