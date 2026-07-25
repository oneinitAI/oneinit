"use client";

import { motion } from "motion/react";

export function Nav() {
  return (
    <motion.nav
      initial={{ opacity: 0, y: -12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
      className="fixed top-0 left-0 right-0 z-50 border-b border-zinc-900/80 bg-zinc-950/80 backdrop-blur-xl"
    >
      <div className="mx-auto flex max-w-[1200px] items-center justify-between px-6 h-16">
        <a href="#" className="flex items-center gap-2 font-mono text-sm font-bold tracking-tight">
          <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-emerald-500 text-zinc-950">
            {"</>"}
          </span>
          oneinit
        </a>
        <div className="flex items-center gap-6">
          <a href="#commands" className="hidden text-sm text-zinc-400 hover:text-zinc-100 transition-colors sm:block">
            Commands
          </a>
          <a href="#install" className="hidden text-sm text-zinc-400 hover:text-zinc-100 transition-colors sm:block">
            Install
          </a>
          <a
            href="https://github.com/BG4JTS/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-lg border border-zinc-800 px-4 py-1.5 text-sm font-medium text-zinc-300 hover:border-zinc-600 hover:text-zinc-100 transition-all active:translate-y-px"
          >
            GitHub
          </a>
        </div>
      </div>
    </motion.nav>
  );
}
