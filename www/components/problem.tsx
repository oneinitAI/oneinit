"use client";

import { useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";
import anime from "animejs";

const ROWS = [
  { old: "Download, Next, Next, Finish", neo: "oneinit install python3.11" },
  { old: "Manually configure pip/npm mirrors", neo: "Auto: Tsinghua / npmmirror" },
  { old: 'rm -rf, hope it\'s clean', neo: "SQLite manifest, 100% rollback" },
  { old: '"What did I install before?"', neo: "oneinit capture → export" },
  { old: "New machine = lost afternoon", neo: "oneinit import backup.tar.gz" },
];

export function Problem() {
  const reduce = useReducedMotion();
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (reduce || !ref.current) return;
    const obs = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) {
        anime({ targets: ".problem-row", translateX: [-60, 0], opacity: [0, 1], delay: anime.stagger(120), duration: 700, easing: "easeOutExpo" });
        obs.disconnect();
      }
    }, { threshold: 0.2 });
    obs.observe(ref.current);
    return () => obs.disconnect();
  }, [reduce]);

  return (
    <section className="relative border-t border-zinc-800 py-32 md:py-40">
      <div className="absolute inset-0 bg-gradient-to-b from-zinc-950 via-zinc-950 to-zinc-900/50" />
      <div ref={ref} className="relative z-10 mx-auto max-w-[1100px] px-6">
        <h2 className="mb-4 text-center font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">
          Why OneInit
        </h2>
        <h3 className="mb-16 text-center text-3xl font-black tracking-tight md:text-5xl">
          Stop wasting hours.<br />
          <span className="bg-gradient-to-r from-emerald-400 to-amber-400 bg-clip-text text-transparent">Start shipping.</span>
        </h3>

        <div className="space-y-2">
          {ROWS.map((row, i) => (
            <div
              key={i}
              className="problem-row group grid grid-cols-[1fr_auto_1fr] items-center gap-4 rounded-xl border border-zinc-800/60 bg-zinc-900/30 p-6 backdrop-blur-sm transition-all hover:border-emerald-500/20 hover:bg-zinc-900/50 md:gap-8 md:p-8"
              style={reduce ? {} : { opacity: 0 }}
            >
              <div className="text-right text-sm text-zinc-500 line-through decoration-zinc-700 md:text-base">
                {row.old}
              </div>
              <div className="font-mono text-lg text-zinc-700 transition-colors group-hover:text-emerald-500 md:text-2xl">
                →
              </div>
              <div className="font-mono text-sm text-zinc-100 md:text-base">
                <span className="text-neon">$ </span>
                {row.neo}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
