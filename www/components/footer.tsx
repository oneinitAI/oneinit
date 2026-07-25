"use client";

import { motion, useReducedMotion } from "motion/react";

export function Footer() {
  const reduce = useReducedMotion();

  return (
    <footer className="border-t border-zinc-900 py-20">
      <div className="mx-auto max-w-[800px] px-6 text-center">
        <motion.h2
          initial={reduce ? undefined : { opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.5 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          className="mb-6 text-3xl font-bold tracking-tight md:text-4xl"
        >
          One command to init
          <br />
          your dev machine.
        </motion.h2>

        <motion.div
          initial={reduce ? undefined : { opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.5 }}
          transition={{ duration: 0.5, delay: 0.15 }}
          className="mb-16 flex flex-col items-center gap-3 sm:flex-row sm:justify-center"
        >
          <a
            href="https://github.com/BG4JTS/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-xl bg-emerald-500 px-6 py-3 font-medium text-zinc-950 transition-all hover:bg-emerald-400 active:translate-y-px active:scale-[0.98]"
          >
            Get Started
          </a>
          <a
            href="https://github.com/BG4JTS/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-xl border border-zinc-800 px-6 py-3 font-medium text-zinc-300 transition-all hover:border-zinc-600 hover:text-zinc-100 active:translate-y-px"
          >
            View on GitHub
          </a>
        </motion.div>

        <div className="flex items-center justify-center gap-6 text-sm text-zinc-600">
          <a
            href="https://github.com/BG4JTS/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-zinc-400"
          >
            GitHub
          </a>
          <a
            href="https://www.npmjs.com/package/oneinit"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-zinc-400"
          >
            npm
          </a>
          <span>GPL-3.0</span>
          <a href="/README_CN.md" className="transition-colors hover:text-zinc-400">
            中文
          </a>
        </div>

        <p className="mt-8 font-mono text-xs text-zinc-700">
          Built with Rust. Powered by 26 tests. No runtime.
        </p>
      </div>
    </footer>
  );
}
