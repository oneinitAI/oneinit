"use client";

import { useEffect, useState, useRef } from "react";
import { useReducedMotion } from "motion/react";
import anime from "animejs";

const CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%&";

export function Hero() {
  const reduce = useReducedMotion();
  const [scrambling, setScrambling] = useState(false);
  const titleRef = useRef<HTMLHeadingElement>(null);
  const scrambleTimer = useRef<ReturnType<typeof setInterval>>();

  useEffect(() => {
    if (reduce || !titleRef.current) return;
    anime({
      targets: ".hero-char",
      opacity: [0, 1],
      translateY: [80, 0],
      rotateX: [90, 0],
      delay: anime.stagger(30, { from: "center" }),
      duration: 900,
      easing: "easeOutExpo",
    });
  }, [reduce]);

  const startScramble = () => {
    if (reduce || !titleRef.current || scrambling) return;
    setScrambling(true);
    const el = titleRef.current;
    const original = el.textContent || "";
    let iterations = 0;
    scrambleTimer.current = setInterval(() => {
      el.textContent = original
        .split("")
        .map((c, i) => {
          if (c === " ") return " ";
          if (iterations > 8 && Math.random() > 0.3) return c;
          return CHARS[Math.floor(Math.random() * CHARS.length)];
        })
        .join("");
      iterations++;
      if (iterations > 15) {
        clearInterval(scrambleTimer.current);
        el.textContent = original;
        setScrambling(false);
      }
    }, 40);
  };

  const stopScramble = () => {
    if (scrambleTimer.current) {
      clearInterval(scrambleTimer.current);
      if (titleRef.current) {
        titleRef.current.textContent = "One command to init your dev machine.";
      }
      setScrambling(false);
    }
  };

  const title = "One command to init your dev machine.";
  const chars = title.split("").map((c, i) => (
    <span key={i} className="hero-char inline-block" style={reduce ? {} : { opacity: 0 }}>
      {c === " " ? "\u00A0" : c}
    </span>
  ));

  return (
    <section className="relative flex min-h-[100dvh] flex-col items-center justify-center overflow-hidden px-6 pt-16">
      {/* Animated background layers */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_rgba(16,185,129,0.12)_0%,_transparent_70%)]" />
        <div className="absolute left-1/2 top-1/2 h-[900px] w-[900px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-gradient-to-br from-neon/8 via-transparent to-crimson/5 blur-3xl animate-pulse" />
        <div className="absolute inset-0 opacity-[0.025]" style={{
          backgroundImage: "linear-gradient(rgba(0,255,136,1) 1px, transparent 1px), linear-gradient(90deg, rgba(0,255,136,1) 1px, transparent 1px)",
          backgroundSize: "60px 60px",
        }} />
      </div>

      <div className="relative z-10 mx-auto max-w-[950px] text-center">
        <div className="mb-8 inline-flex items-center gap-3 rounded-full border border-neon/30 bg-neon/5 px-4 py-2 backdrop-blur-md transition-all hover:border-neon/50">
          <span className="h-2 w-2 animate-pulse rounded-full bg-neon shadow-[0_0_10px_rgba(0,255,136,0.7)]" />
          <span className="font-mono text-xs uppercase tracking-[0.2em] text-neon">
            AI-First Environment Initializer
          </span>
        </div>

        <h1
          ref={titleRef}
          className="mb-6 text-5xl font-black leading-[1.02] tracking-tighter transition-colors md:text-7xl lg:text-[84px]"
          data-clickable
          onMouseEnter={startScramble}
          onMouseLeave={stopScramble}
        >
          {chars}
        </h1>

        <div className="mb-8 flex items-center justify-center gap-4">
          <div className="h-px w-16 bg-gradient-to-r from-transparent to-neon" />
          <span className="chromatic font-mono text-sm text-neon">
            17 commands · 7 detectors · 26 tests · 7.3MB
          </span>
          <div className="h-px w-16 bg-gradient-to-l from-transparent to-neon" />
        </div>

        <div className="flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <a
            href="#install"
            data-clickable
            className="group relative overflow-hidden rounded-lg bg-neon px-8 py-4 font-bold text-zinc-950 transition-all hover:bg-emerald-400 hover:shadow-[0_0_60px_rgba(0,255,136,0.5)] active:scale-[0.97]"
          >
            <span className="relative z-10">Get Started</span>
            <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/30 to-transparent group-hover:translate-x-full transition-transform duration-700" />
          </a>
          <div
            data-clickable
            className="flex cursor-pointer items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900/80 px-5 py-4 font-mono text-sm text-zinc-300 backdrop-blur-md transition-all hover:border-neon/50 hover:text-neon active:scale-[0.97]"
            onClick={() => navigator.clipboard.writeText("npm i -g oneinit")}
          >
            <span className="text-neon">$</span>
            npm i -g oneinit
          </div>
        </div>
      </div>

      <div className="absolute bottom-8 left-1/2 -translate-x-1/2">
        <div className="flex flex-col items-center gap-2 text-zinc-700">
          <span className="font-mono text-[10px] uppercase tracking-[0.3em]">Scroll</span>
          <div className="h-10 w-px bg-gradient-to-b from-neon to-transparent animate-pulse" />
        </div>
      </div>
    </section>
  );
}
