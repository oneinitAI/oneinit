"use client";

import { motion, useReducedMotion } from "motion/react";

const TILES = [
  {
    cmd: "install",
    desc: "python3.11, node20",
    detail: "Download, verify, extract, mirror, PATH",
    span: "md:col-span-2",
  },
  {
    cmd: "uninstall",
    desc: "full rollback",
    detail: "PATH, configs, files, manifest",
    span: "",
  },
  {
    cmd: "capture",
    desc: "7 detectors",
    detail: "Python, Node, Git, Rust, Go, Java, Docker",
    span: "",
  },
  {
    cmd: "export",
    desc: "tar.gz backup",
    detail: "Full environment, portable",
    span: "",
  },
  {
    cmd: "tui",
    desc: "interactive",
    detail: "Dual-pane menu, keyboard-driven",
    span: "md:col-span-2",
  },
  {
    cmd: "search",
    desc: "builtin + remote",
    detail: "3-tier: compiled, local, registry",
    span: "",
  },
  {
    cmd: "publish",
    desc: "npm-like",
    detail: "YAML recipe to community registry",
    span: "",
  },
  {
    cmd: "doctor",
    desc: "health check",
    detail: "PATH, manifest, disk, cache",
    span: "md:col-span-2",
  },
];

export function CommandShowcase() {
  const reduce = useReducedMotion();

  return (
    <section id="commands" className="border-t border-zinc-900 py-24 md:py-32">
      <div className="mx-auto max-w-[1200px] px-6">
        <motion.h2
          initial={reduce ? undefined : { opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.5 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          className="mb-3 text-3xl font-bold tracking-tight md:text-4xl"
        >
          17 commands. One binary.
        </motion.h2>
        <p className="mb-12 max-w-[480px] text-zinc-400">
          Everything you need to bootstrap, configure, and migrate a development
          environment.
        </p>

        <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
          {TILES.map((tile, i) => (
            <motion.div
              key={tile.cmd}
              initial={reduce ? undefined : { opacity: 0, scale: 0.95 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true, amount: 0.2 }}
              transition={{
                duration: 0.4,
                delay: i * 0.05,
                ease: [0.16, 1, 0.3, 1],
              }}
              whileHover={reduce ? undefined : { borderColor: "rgba(16,185,129,0.3)" }}
              className={`group rounded-xl border border-zinc-800 bg-zinc-900/30 p-5 transition-colors ${tile.span}`}
            >
              <div className="flex items-baseline gap-2">
                <span className="font-mono text-sm text-emerald-500">$</span>
                <span className="font-mono text-base font-medium text-zinc-100">
                  oneinit {tile.cmd}
                </span>
              </div>
              <div className="mt-2 text-sm text-zinc-400">{tile.desc}</div>
              <div className="mt-1 text-xs text-zinc-600">{tile.detail}</div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
