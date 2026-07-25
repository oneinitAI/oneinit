"use client";

import { useEffect } from "react";
import { useReducedMotion } from "motion/react";
import anime from "animejs";

export function Hero() {
  const reduce = useReducedMotion();

  useEffect(() => {
    if (reduce) return;
    anime({
      targets: ".hero-char",
      opacity: [0, 1],
      translateY: [60, 0],
      rotateX: [90, 0],
      delay: anime.stagger(40, { from: "center" }),
      duration: 800,
      easing: "easeOutExpo",
    });
    anime({
      targets: ".hero-line",
      opacity: [0, 1],
      translateY: [30, 0],
      delay: anime.stagger(150),
      duration: 600,
      easing: "easeOutCubic",
    });
  }, [reduce]);

  const title = "One command to init your dev machine.";
  const chars = title.split("").map((c, i) => (
    <span key={i} className="hero-char inline-block" style={reduce ? {} : { opacity: 0 }}>
      {c === " " ? "\u00A0" : c}
    </span>
  ));

  return (
    <section className="relative flex min-h-[100dvh] flex-col items-center justify-center overflow-hidden px-6 pt-16">
      {/* Animated background */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_rgba(16,185,129,0.15)_0%,_transparent_70%)]" />
        <div className="absolute left-1/2 top-1/2 h-[800px] w-[800px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-gradient-to-br from-emerald-500/10 via-transparent to-crimson/5 blur-3xl" />
        {/* Grid lines */}
        <div className="absolute inset-0 opacity-[0.03]" style={{
          backgroundImage: "linear-gradient(rgba(16,185,129,1) 1px, transparent 1px), linear-gradient(90deg, rgba(16,185,129,1) 1px, transparent 1px)",
          backgroundSize: "80px 80px",
        }} />
      </div>

      <div className="relative z-10 mx-auto max-w-[900px] text-center">
        {/* Eyebrow */}
        <div className="hero-line mb-8 inline-flex items-center gap-3 rounded-full border border-emerald-500/30 bg-emerald-500/5 px-4 py-2 backdrop-blur-md">
          <span className="h-2 w-2 animate-pulse rounded-full bg-neon shadow-[0_0_8px_rgba(0,255,136,0.6)]" />
          <span className="font-mono text-xs uppercase tracking-[0.2em] text-emerald-400">
            AI-First Environment Initializer
          </span>
        </div>

        {/* Massive typography */}
        <h1 className="mb-6 text-5xl font-black leading-[1.02] tracking-tighter md:text-7xl lg:text-[82px]">
          {chars}
        </h1>

        {/* Glitch accent line */}
        <div className="hero-line mb-8 flex items-center justify-center gap-4">
          <div className="h-px w-16 bg-gradient-to-r from-transparent to-emerald-500" />
          <span className="font-mono text-sm text-emerald-500 chromatic">
            17 commands · 7 detectors · 26 tests · 7.3MB
          </span>
          <div className="h-px w-16 bg-gradient-to-l from-transparent to-emerald-500" />
        </div>

        {/* CTAs */}
        <div className="hero-line flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a
            href="#install"
            data-clickable
            className="group relative overflow-hidden rounded-lg bg-emerald-500 px-8 py-4 font-bold text-zinc-950 transition-all hover:bg-neon hover:shadow-[0_0_40px_rgba(0,255,136,0.4)] active:scale-[0.97]"
          >
            <span className="relative z-10">Get Started</span>
            <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/20 to-transparent group-hover:translate-x-full transition-transform duration-700" />
          </a>
          <div
            data-clickable
            className="flex cursor-pointer items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900/80 px-5 py-4 font-mono text-sm text-zinc-300 backdrop-blur-md transition-all hover:border-emerald-500/50 hover:text-emerald-400"
            onClick={() => navigator.clipboard.writeText("npm i -g oneinit")}
          >
            <span className="text-neon">$</span>
            npm i -g oneinit
          </div>
        </div>
      </div>

      {/* Scroll indicator */}
      <div className="absolute bottom-8 left-1/2 -translate-x-1/2">
        <div className="flex flex-col items-center gap-2 text-zinc-600">
          <span className="font-mono text-[10px] uppercase tracking-[0.3em]">Scroll</span>
          <div className="h-8 w-px bg-gradient-to-b from-emerald-500 to-transparent animate-pulse" />
        </div>
      </div>
    </section>
  );
}
