"use client";

import { useEffect, useRef } from "react";
import { motion, useReducedMotion } from "motion/react";
import anime from "animejs";
import { Terminal } from "./terminal";
import { MagneticButton } from "./magnetic-button";

export function Hero() {
  const reduce = useReducedMotion();
  const titleRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    if (reduce || !titleRef.current) return;

    anime({
      targets: titleRef.current,
      translateY: [40, 0],
      opacity: [0, 1],
      filter: ["blur(8px)", "blur(0px)"],
      duration: 900,
      easing: "easeOutExpo",
    });
  }, [reduce]);

  return (
    <section className="relative flex min-h-[100dvh] items-center overflow-hidden pt-16">
      {/* Radial glow */}
      <div className="pointer-events-none absolute inset-0 overflow-hidden">
        <div className="absolute left-1/4 top-1/4 h-[600px] w-[600px] animate-pulse rounded-full bg-emerald-500/6 blur-[150px]" style={{ animationDuration: "8s" }} />
        <div className="absolute right-1/4 top-1/2 h-[400px] w-[400px] rounded-full bg-teal-500/4 blur-[120px]" style={{ animationDelay: "2s", animationDuration: "6s" }} />
      </div>

      <div className="relative z-10 mx-auto grid w-full max-w-[1200px] grid-cols-1 gap-12 px-6 lg:grid-cols-[1.1fr_0.9fr] lg:gap-8">
        {/* Left: Copy */}
        <div className="flex flex-col justify-center">
          <motion.div
            initial={reduce ? undefined : { opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
            className="mb-5 inline-flex w-fit items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-3 py-1 backdrop-blur-sm"
          >
            <span className="h-2 w-2 animate-pulse rounded-full bg-emerald-500" />
            <span className="font-mono text-xs text-emerald-400">v0.1.0 beta</span>
          </motion.div>

          <h1
            ref={titleRef}
            className="text-4xl font-bold leading-[1.05] tracking-tight md:text-5xl lg:text-6xl"
            style={reduce ? {} : { opacity: 0 }}
          >
            One command to init
            <br />
            your{" "}
            <span className="bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">
              dev
            </span>{" "}
            machine.
          </h1>

          <motion.p
            initial={reduce ? undefined : { opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
            className="mt-5 max-w-[480px] text-base leading-relaxed text-zinc-400 md:text-lg"
          >
            The first tool to install on a new computer. Python, Node.js,
            Rust, Go - installed, mirrored, PATH-configured. All in one line.
          </motion.p>

          <motion.div
            initial={reduce ? undefined : { opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.55 }}
            className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center"
          >
            <MagneticButton href="#install" variant="primary">
              Get Started
            </MagneticButton>
            <div className="flex items-center gap-2 rounded-xl border border-zinc-700 bg-zinc-900/50 px-4 py-3 font-mono text-sm text-zinc-300 backdrop-blur-sm">
              <span className="text-emerald-500">$</span>
              npm i -g oneinit
            </div>
          </motion.div>
        </div>

        {/* Right: Terminal */}
        <motion.div
          initial={reduce ? undefined : { opacity: 0, scale: 0.92 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.8, delay: 0.35, ease: [0.16, 1, 0.3, 1] }}
          className="flex items-center justify-center"
        >
          <Terminal />
        </motion.div>
      </div>
    </section>
  );
}
