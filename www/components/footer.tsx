"use client";export function Footer(){return(
<footer className="border-t border-white/[0.04] py-20 text-center"><div className="mx-auto max-w-[600px] px-6">
<img src="/logo.png" alt="OneInit" className="mx-auto h-10 w-auto mb-6 opacity-80" />
<h2 className="text-3xl font-bold tracking-tight text-white md:text-5xl">One command to <span className="text-emerald-500">init your dev machine</span>.</h2>
<p className="mt-4 text-zinc-500">17 commands. 7 detectors. 26 tests. 7.3MB. Zero runtime.</p>
<div className="mt-8 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
<a href="#install" className="rounded-xl bg-emerald-600 px-8 py-3.5 font-bold text-white transition-all hover:bg-emerald-500 hover:shadow-lg hover:shadow-emerald-600/20 active:scale-[0.98]">Get Started</a>
<a href="https://github.com/BG4JTS/oneinit" target="_blank" rel="noopener noreferrer" className="glass glass-hover rounded-xl px-8 py-3.5 font-bold text-zinc-300 transition-all">View on GitHub</a>
</div>
<div className="mt-12 flex items-center justify-center gap-6 text-sm text-zinc-600">
<a href="https://github.com/BG4JTS/oneinit" className="hover:text-zinc-400">GitHub</a><a href="https://www.npmjs.com/package/oneinit" className="hover:text-zinc-400">npm</a><span>GPL-3.0</span>
</div><p className="mt-4 font-mono text-xs text-zinc-700">Built with Rust · No runtime · Single binary</p><p className="mt-1 font-mono text-xs text-zinc-800">&copy; {new Date().getFullYear()} BG4JTS. All rights reserved.</p></div></footer>)}