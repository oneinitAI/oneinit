"use client";

import { useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";
import anime from "animejs";

const CAPS = [
  {
    num: "01",
    title: "Auto Mirror Config",
    body: "pip automatically uses Tsinghua. npm uses npmmirror. No config files. No searching for registry URLs.",
    code: `[global]
index-url = https://pypi.tuna.tsinghua.edu.cn/simple
trusted-host = pypi.tuna.tsinghua.edu.cn`,
    color: "emerald",
  },
  {
    num: "02",
    title: "7 Language Detectors",
    body: "Python, Node.js, Git, Rust, Go, Java, Docker. Plus custom detectors. Scan any machine, export the blueprint.",
    code: `$ oneinit capture
[OK] python 3.13.2 (120 packages)
[OK] node 24.13.0 · git 2.46.0
[OK] rust 1.94.0 · go 1.25.0
[OK] java 21.0.11`,
    color: "amber",
  },
  {
    num: "03",
    title: "Community Registry",
    body: "Publish YAML recipes. Others install with one command. Like npm, but for dev tools. Versioned, reviewed, secure.",
    code: `name: node20
version: "20.18.1"
platforms:
  windows:
    url: "https://nodejs.org/..."
    sha256: "56e5aacd..."
    install_type: "zip_extract"
post_install: ...`,
    color: "crimson",
  },
  {
    num: "04",
    title: "Full Environment Migration",
    body: "Export your entire setup as tar.gz. Import on a new machine. Tools, configs, packages — everything restored.",
    code: `$ oneinit export -o backup.tar.gz
  --include-envs
$ oneinit import backup.tar.gz`,
    color: "emerald",
  },
];

export function Capabilities() {
  const reduce = useReducedMotion();
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (reduce || !ref.current) return;
    const obs = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) {
        anime({
          targets: ".cap-section",
          translateY: [80, 0],
          opacity: [0, 1],
          delay: anime.stagger(200),
          duration: 800,
          easing: "easeOutExpo",
        });
        obs.disconnect();
      }
    }, { threshold: 0.1 });
    obs.observe(ref.current);
    return () => obs.disconnect();
  }, [reduce]);

  const colorMap: Record<string, string> = {
    emerald: "border-emerald-500/20",
    amber: "border-amber-500/20",
    crimson: "border-crimson/20",
  };

  return (
    <section className="relative border-t border-zinc-800 py-32 md:py-40 overflow-hidden">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_left,_rgba(255,170,0,0.05)_0%,_transparent_60%)]" />
      <div ref={ref} className="relative z-10 mx-auto max-w-[1200px] px-6">
        {CAPS.map((cap) => (
          <div
            key={cap.num}
            className={`cap-section mb-20 border-l-2 ${colorMap[cap.color] || "border-zinc-800"} pl-6 md:pl-10`}
            style={reduce ? {} : { opacity: 0 }}
          >
            <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_1.2fr] lg:gap-12">
              <div>
                <div className="mb-3 font-mono text-6xl font-black tracking-tighter text-zinc-800">
                  {cap.num}
                </div>
                <h3 className="mb-3 text-2xl font-bold tracking-tight md:text-3xl">{cap.title}</h3>
                <p className="max-w-[400px] leading-relaxed text-zinc-400">{cap.body}</p>
              </div>
              <div className="overflow-hidden rounded-xl border border-zinc-800/60 bg-zinc-950/80">
                <div className="flex items-center gap-1.5 border-b border-zinc-800/60 px-4 py-2.5">
                  <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                  <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                  <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                </div>
                <pre className="terminal-scroll overflow-x-auto p-5 font-mono text-[13px] leading-relaxed text-zinc-300">
                  <code>{cap.code}</code>
                </pre>
              </div>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
