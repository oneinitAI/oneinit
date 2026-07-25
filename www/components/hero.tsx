"use client";
import { motion } from "motion/react";
import { Terminal } from "./terminal";

export function Hero() {
  return (
    <section className="relative flex min-h-[100dvh] items-center overflow-hidden pt-16">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute -top-20 left-1/3 h-[500px] w-[700px] rounded-full bg-emerald-600/[0.04] blur-[140px]" />
      </div>

      <div className="relative z-10 mx-auto grid max-w-[1200px] grid-cols-1 gap-12 px-6 lg:grid-cols-[1.1fr_0.9fr] lg:gap-8 w-full">
        <div className="flex flex-col justify-center">
          <motion.div initial={{ opacity:0,y:20 }} animate={{ opacity:1,y:0 }} transition={{ duration:0.5,delay:0.1 }}
            className="mb-6 inline-flex w-fit items-center gap-2 rounded-full border border-emerald-600/20 bg-emerald-600/5 px-4 py-1.5">
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
            <span className="font-mono text-xs tracking-widest text-emerald-500">AI-FIRST ENVIRONMENT INITIALIZER</span>
          </motion.div>

          <motion.h1 initial={{ opacity:0,y:30 }} animate={{ opacity:1,y:0 }} transition={{ duration:0.6,delay:0.2 }}
            className="text-5xl font-bold leading-[1.04] tracking-tight text-white md:text-6xl lg:text-[78px]">
            One command<br />
            to init your<br />
            <span className="text-emerald-500">dev machine.</span>
          </motion.h1>

          <motion.p initial={{ opacity:0,y:20 }} animate={{ opacity:1,y:0 }} transition={{ duration:0.5,delay:0.35 }}
            className="mt-5 max-w-[480px] text-base leading-relaxed text-zinc-400 md:text-lg">
            Python, Node.js, Rust, Go — installed, mirrored, PATH-configured. All in one line. Zero sudo. SQLite manifest for clean rollback.
          </motion.p>

          <motion.div initial={{ opacity:0,y:16 }} animate={{ opacity:1,y:0 }} transition={{ duration:0.5,delay:0.5 }}
            className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
            <a href="#install" className="rounded-xl bg-emerald-600 px-7 py-3.5 font-bold text-white transition-all hover:bg-emerald-500 hover:shadow-lg hover:shadow-emerald-600/20 active:scale-[0.98]">
              Get Started
            </a>
            <button onClick={() => navigator.clipboard.writeText("npm i -g oneinit")}
              className="glass rounded-xl px-5 py-3.5 font-mono text-sm text-zinc-300 cursor-pointer glass-hover transition-all">
              <span className="text-emerald-500">$</span> npm i -g oneinit
            </button>
          </motion.div>
        </div>

        <motion.div initial={{ opacity:0,scale:0.95 }} animate={{ opacity:1,scale:1 }} transition={{ duration:0.7,delay:0.4 }}
          className="flex items-center justify-center"><Terminal /></motion.div>
      </div>
    </section>
  );
}
