"use client";
import { useEffect, useRef } from "react";
import { motion } from "motion/react";
import { Terminal } from "./terminal";

export function Hero() {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current; if (!el) return;
    const onMove = (e: MouseEvent) => {
      const x = (e.clientX / window.innerWidth - 0.5) * 20;
      const y = (e.clientY / window.innerHeight - 0.5) * 20;
      el.style.setProperty("--mx", `${x}px`); el.style.setProperty("--my", `${y}px`);
    };
    window.addEventListener("mousemove", onMove);
    return () => window.removeEventListener("mousemove", onMove);
  }, []);

  return (
    <section className="relative flex min-h-[100dvh] items-center overflow-hidden pt-16">
      <div ref={ref} className="pointer-events-none absolute inset-0"
        style={{ transform: "translate(var(--mx, 0px), var(--my, 0px))" }}>
        <div className="absolute -top-40 left-1/2 h-[600px] w-[800px] -translate-x-1/2 rounded-full bg-gradient-to-br from-cyan/15 via-purple/10 to-transparent blur-[120px] pulse-glow" />
      </div>

      <div className="relative z-10 mx-auto grid max-w-[1200px] grid-cols-1 gap-12 px-6 lg:grid-cols-[1.1fr_0.9fr] lg:gap-8 w-full">
        <div className="flex flex-col justify-center">
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.6, delay: 0.1 }}
            className="mb-6 inline-flex w-fit items-center gap-2 rounded-full border border-[rgba(0,240,255,0.2)] bg-[rgba(0,240,255,0.05)] px-4 py-1.5 backdrop-blur-sm">
            <span className="h-2 w-2 rounded-full bg-cyan animate-pulse shadow-[0_0_8px_rgba(0,240,255,0.6)]" />
            <span className="font-mono text-xs tracking-widest text-cyan">AI-FIRST ENVIRONMENT INITIALIZER</span>
          </motion.div>

          <motion.h1 initial={{ opacity: 0, y: 30 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.7, delay: 0.2 }}
            className="text-5xl font-bold leading-[1.05] tracking-tight md:text-6xl lg:text-[76px]">
            One command to init<br />
            your <span className="text-gradient">dev machine</span>.
          </motion.h1>

          <motion.p initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.6, delay: 0.35 }}
            className="mt-5 max-w-[500px] text-base leading-relaxed text-zinc-400 md:text-lg">
            The first tool to install on a new computer. Python, Node.js, Rust, Go — installed, mirrored, PATH-configured. Zero sudo. All in one line.
          </motion.p>

          <motion.div initial={{ opacity: 0, y: 16 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.6, delay: 0.5 }}
            className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
            <a href="#install" className="group relative overflow-hidden rounded-xl bg-gradient-to-r from-cyan to-purple px-7 py-3.5 font-bold text-white shadow-lg shadow-purple/25 transition-all hover:shadow-cyan/30 hover:scale-[1.02] active:scale-[0.98]">
              <span className="relative z-10">Get Started</span>
              <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/10 to-transparent group-hover:translate-x-full transition-transform duration-700" />
            </a>
            <div className="flex items-center gap-2 rounded-xl glass px-5 py-3.5 font-mono text-sm text-zinc-300 cursor-pointer hover:border-cyan/20 transition-all"
              onClick={() => navigator.clipboard.writeText("npm i -g oneinit")}>
              <span className="text-cyan">$</span> npm i -g oneinit
            </div>
          </motion.div>
        </div>

        <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.8, delay: 0.4 }}
          className="flex items-center justify-center">
          <Terminal />
        </motion.div>
      </div>
    </section>
  );
}
