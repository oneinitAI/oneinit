"use client";

import { motion, useReducedMotion } from "motion/react";

const CAPS = [
  {
    title: "Auto mirror configuration",
    body: "pip automatically uses Tsinghua mirror. npm uses npmmirror. No manual setup, no config files to write.",
    code: `[global]
index-url = https://pypi.tuna.tsinghua.edu.cn/simple
trusted-host = pypi.tuna.tsinghua.edu.cn`,
    lang: "ini",
  },
  {
    title: "7 language detectors",
    body: "Scan your current machine for Python, Node.js, Git, Rust, Go, Java, Docker. Plus custom detectors via scan_config.yaml.",
    code: `$ oneinit capture
[OK] python 3.13.2 (120 global packages)
[OK] node 24.13.0 (9 global packages)
[OK] git 2.46.0
[OK] rust 1.94.0
[OK] go 1.25.0
[OK] java 21.0.11
[--] docker not detected`,
    lang: "bash",
  },
  {
    title: "Community recipe registry",
    body: "Write a YAML recipe, publish it. Others install with one command. Like npm, but for dev tools.",
    code: `name: node20
version: "20.18.1"
platforms:
  windows:
    url: "https://nodejs.org/dist/..."
    sha256: "56e5aacd..."
    install_type: "zip_extract"
post_install:
  config_files:
    - path: ".npmrc"
      template: "registry={{mirror_npm}}"`,
    lang: "yaml",
  },
  {
    title: "Full environment migration",
    body: "Export your entire dev setup as a portable tar.gz. Import on a new machine. Tools, configs, package lists, all restored.",
    code: `# Old machine
oneinit export -o backup.tar.gz --include-envs

# New machine
oneinit import backup.tar.gz --dry-run
oneinit import backup.tar.gz`,
    lang: "bash",
  },
];

export function Capabilities() {
  const reduce = useReducedMotion();

  return (
    <section className="border-t border-zinc-900 py-24 md:py-32">
      <div className="mx-auto max-w-[1200px] px-6">
        {CAPS.map((cap, i) => {
          const isEven = i % 2 === 0;
          return (
            <div
              key={i}
              className={`grid grid-cols-1 items-center gap-12 py-16 md:grid-cols-2 ${
                i < CAPS.length - 1 ? "border-b border-zinc-900" : ""
              }`}
            >
              {/* Text */}
              <motion.div
                initial={reduce ? undefined : { opacity: 0, y: 24 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, amount: 0.3 }}
                transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
                className={isEven ? "md:order-1" : "md:order-2"}
              >
                <h3 className="mb-3 text-2xl font-bold tracking-tight">
                  {cap.title}
                </h3>
                <p className="max-w-[420px] leading-relaxed text-zinc-400">
                  {cap.body}
                </p>
              </motion.div>

              {/* Code block */}
              <motion.div
                initial={reduce ? undefined : { opacity: 0, scale: 0.96 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true, amount: 0.3 }}
                transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
                className={isEven ? "md:order-2" : "md:order-1"}
              >
                <div className="overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900/50">
                  <div className="flex items-center gap-1.5 border-b border-zinc-800 px-4 py-2">
                    <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                    <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                    <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                    <span className="ml-2 font-mono text-xs text-zinc-600">
                      {cap.lang}
                    </span>
                  </div>
                  <pre className="terminal-scroll overflow-x-auto p-4 font-mono text-[13px] leading-relaxed text-zinc-300">
                    <code>{cap.code}</code>
                  </pre>
                </div>
              </motion.div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
