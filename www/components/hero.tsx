"use client";

import { motion, useReducedMotion } from "motion/react";
import { Terminal } from "./terminal";

export function Hero() {
  const reduce = useReducedMotion();

  return (
    <section className="relative flex min-h-[100dvh] items-center overflow-hidden pt-16">
      {/* Subtle radial glow */}
      <div className="pointer-events-none absolute inset-0 overflow-hidden">
        <div className="absolute left-1/4 top-1/3 h-[500px] w-[500px] rounded-full bg-emerald-500/8 blur-[120px]" />
      </div>

      <div className="mx-auto grid w-full max-w-[1200px] grid-cols-1 gap-12 px-6 lg:grid-cols-[1.1fr_0.9fr] lg:gap-8">
        {/* Left: Copy */}
        <div className="flex flex-col justify-center">
          <motion.div
            initial={reduce ? undefined : { opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
            className="mb-5 inline-flex w-fit items-center gap-2 rounded-full border border-zinc-800 bg-zinc-900/50 px-3 py-1"
          >
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
            <span className="font-mono text-xs text-zinc-400">v0.1.0 beta</span>
          </motion.div>

          <motion.h1
            initial={reduce ? undefined : { opacity: 0, y: 24, filter: "blur(8px)" }}
            animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
            transition={{ duration: 0.7, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
            className="text-4xl font-bold leading-[1.05] tracking-tight md:text-5xl lg:text-6xl"
          >
            One command to init
            <br />
            your dev machine.
          </motion.h1>

          <motion.p
            initial={reduce ? undefined : { opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.25, ease: [0.16, 1, 0.3, 1] }}
            className="mt-5 max-w-[480px] text-base leading-relaxed text-zinc-400 md:text-lg"
          >
            The first tool to install on a new computer. Python, Node.js, Rust,
            Go, installed, mirrored, PATH-configured. All in one line.
          </motion.p>

          <motion.div
            initial={reduce ? undefined : { opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.4 }}
            className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center"
          >
            <a
              href="#install"
              className="rounded-xl bg-emerald-500 px-6 py-3 text-center font-medium text-zinc-950 transition-all hover:bg-emerald-400 active:translate-y-px active:scale-[0.98]"
            >
              Get Started
            </a>
            <div className="flex items-center gap-2 rounded-xl border border-zinc-800 bg-zinc-900/50 px-4 py-3 font-mono text-sm text-zinc-400">
              <span className="text-emerald-500">$</span>
              npm i -g oneinit
            </div>
          </motion.div>
        </div>

        {/* Right: Terminal */}
        <motion.div
          initial={reduce ? undefined : { opacity: 0, scale: 0.96 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.8, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
          className="flex items-center justify-center"
        >
          <Terminal />
        </motion.div>
      </div>
    </section>
  );
}
