"use client";

import { useEffect, useRef, useCallback } from "react";
import { useReducedMotion } from "motion/react";
import anime from "animejs";

const TILES = [
  { cmd: "install", sub: "python3.11, node20", tag: "package" },
  { cmd: "capture", sub: "7 language detectors", tag: "scan" },
  { cmd: "export", sub: "portable tar.gz", tag: "migrate" },
  { cmd: "search", sub: "builtin + community + remote", tag: "discover" },
  { cmd: "publish", sub: "YAML to registry", tag: "share" },
  { cmd: "doctor", sub: "health check", tag: "maintain" },
  { cmd: "uninstall", sub: "full rollback", tag: "clean" },
  { cmd: "tui", sub: "interactive menu", tag: "ui" },
];

export function CommandShowcase() {
  const reduce = useReducedMotion();
  const gridRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (reduce || !gridRef.current) return;
    const obs = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) {
        anime({ targets: ".cmd-tile", scale: [0.6, 1], opacity: [0, 1], delay: anime.stagger(50), duration: 500, easing: "easeOutBack" });
        obs.disconnect();
      }
    }, { threshold: 0.1 });
    obs.observe(gridRef.current);
    return () => obs.disconnect();
  }, [reduce]);

  const handleTilt = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (reduce) return;
    const el = e.currentTarget;
    const rect = el.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width - 0.5;
    const y = (e.clientY - rect.top) / rect.height - 0.5;
    el.style.transform = `perspective(1000px) rotateY(${x * 8}deg) rotateX(${-y * 8}deg) translateZ(10px)`;
  }, [reduce]);

  const handleTiltLeave = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    e.currentTarget.style.transform = "perspective(1000px) rotateY(0) rotateX(0) translateZ(0)";
  }, []);

  return (
    <section className="relative border-t border-zinc-800 py-32 md:py-40 overflow-hidden">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_rgba(16,185,129,0.06)_0%,_transparent_60%)]" />
      <div className="relative z-10 mx-auto max-w-[1200px] px-6">
        <h2 className="mb-2 text-center font-mono text-xs uppercase tracking-[0.3em] text-amber-500">
          Commands
        </h2>
        <h3 className="mb-4 text-center text-3xl font-black tracking-tight md:text-5xl">
          17 commands.{" "}
          <span className="chromatic">One binary.</span>
        </h3>
        <p className="mb-16 text-center text-zinc-500">
          Install, scan, migrate, publish, maintain. Everything a dev needs.
        </p>

        <div ref={gridRef} className="grid grid-cols-2 gap-3 md:grid-cols-4 md:gap-4">
          {TILES.map((tile) => (
            <div
              key={tile.cmd}
              className="cmd-tile tilt-card group relative cursor-default rounded-xl border border-zinc-800/60 bg-zinc-900/40 backdrop-blur-sm transition-colors duration-300 hover:border-emerald-500/20"
              data-clickable
              onMouseMove={handleTilt}
              onMouseLeave={handleTiltLeave}
              style={reduce ? {} : { opacity: 0 }}
            >
              <div className="relative z-10 p-5 md:p-6">
                <div className="mb-2 font-mono text-[10px] uppercase tracking-wider text-zinc-600">{tile.tag}</div>
                <div className="mb-1 font-mono text-lg font-bold text-zinc-100 transition-colors group-hover:text-neon md:text-xl">
                  oneinit {tile.cmd}
                </div>
                <div className="text-xs text-zinc-500">{tile.sub}</div>
              </div>
              {/* Hover gradient */}
              <div className="pointer-events-none absolute inset-0 rounded-xl bg-gradient-to-br from-emerald-500/0 via-transparent to-emerald-500/0 opacity-0 transition-opacity duration-500 group-hover:opacity-100 group-hover:from-emerald-500/5 group-hover:to-amber-500/5" />
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
