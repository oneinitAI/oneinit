"use client";

import { useEffect, useRef, useState } from "react";
import { motion, useReducedMotion } from "motion/react";
import anime from "animejs";

const TILES = [
  { cmd: "install", desc: "python3.11, node20", detail: "Download, verify, extract, mirror, PATH", span: "md:col-span-2" },
  { cmd: "uninstall", desc: "full rollback", detail: "PATH, configs, files, manifest", span: "" },
  { cmd: "capture", desc: "7 detectors", detail: "Python, Node, Git, Rust, Go, Java, Docker", span: "" },
  { cmd: "export", desc: "tar.gz backup", detail: "Full environment, portable", span: "" },
  { cmd: "tui", desc: "interactive", detail: "Dual-pane menu, keyboard-driven", span: "md:col-span-2" },
  { cmd: "search", desc: "builtin + remote", detail: "3-tier: compiled, local, registry CDN", span: "" },
  { cmd: "publish", desc: "npm-like", detail: "YAML recipe to community registry", span: "" },
  { cmd: "doctor", desc: "health check", detail: "PATH, manifest, disk, cache", span: "md:col-span-2" },
];

export function CommandShowcase() {
  const reduce = useReducedMotion();
  const gridRef = useRef<HTMLDivElement>(null);
  const [triggered, setTriggered] = useState(false);

  useEffect(() => {
    if (reduce) return;
    const observer = new IntersectionObserver(
      ([entry]) => { if (entry.isIntersecting) { setTriggered(true); observer.disconnect(); } },
      { threshold: 0.15 }
    );
    if (gridRef.current) observer.observe(gridRef.current);
    return () => observer.disconnect();
  }, [reduce]);

  useEffect(() => {
    if (!triggered || reduce || !gridRef.current) return;
    const tiles = gridRef.current.querySelectorAll(".command-tile");
    anime({
      targets: tiles,
      translateY: [40, 0],
      opacity: [0, 1],
      scale: [0.92, 1],
      delay: anime.stagger(60, { easing: "easeOutExpo" }),
      duration: 600,
      easing: "easeOutExpo",
    });
  }, [triggered, reduce]);

  return (
    <section id="commands" className="border-t border-zinc-900 py-24 md:py-32">
      <div className="mx-auto max-w-[1200px] px-6">
        <motion.h2
          initial={reduce ? undefined : { opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.5 }}
          transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
          className="mb-3 text-3xl font-bold tracking-tight md:text-4xl"
        >
          17 commands. One binary.
        </motion.h2>
        <p className="mb-12 max-w-[480px] text-zinc-400">
          Everything you need to bootstrap, configure, and migrate a development environment.
        </p>

        <div ref={gridRef} className="grid grid-cols-1 gap-3 md:grid-cols-3">
          {TILES.map((tile, i) => (
            <div
              key={tile.cmd}
              className={`command-tile group relative cursor-default rounded-xl border border-zinc-800/80 bg-zinc-900/30 p-5 backdrop-blur-sm transition-all duration-300 hover:border-emerald-500/30 hover:bg-zinc-900/50 hover:shadow-lg hover:shadow-emerald-500/5 ${tile.span}`}
              style={triggered && !reduce ? { opacity: 0 } : {}}
            >
              {/* Hover glow */}
              <div className="pointer-events-none absolute inset-0 rounded-xl bg-gradient-to-br from-emerald-500/0 via-transparent to-emerald-500/0 opacity-0 transition-opacity duration-500 group-hover:opacity-100 group-hover:from-emerald-500/5 group-hover:to-emerald-500/5" />

              <div className="relative z-10">
                <div className="flex items-baseline gap-2">
                  <span className="font-mono text-sm text-emerald-500">$</span>
                  <span className="font-mono text-base font-medium text-zinc-100 transition-colors group-hover:text-emerald-400">
                    oneinit {tile.cmd}
                  </span>
                </div>
                <div className="mt-2 text-sm text-zinc-400">{tile.desc}</div>
                <div className="mt-1 text-xs text-zinc-600">{tile.detail}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
