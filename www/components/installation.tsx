"use client";

import { useState } from "react";
import { useReducedMotion } from "motion/react";

const TABS = [
  {
    id: "shell",
    label: "Shell (zero deps)",
    code: "curl -fsSL https://raw.githubusercontent.com/BG4JTS/oneinit/main/install.sh | sh",
    note: "No prerequisites. Auto-detects your OS and architecture.",
  },
  {
    id: "npm",
    label: "npm",
    code: "npm install -g oneinit",
    note: "Requires Node.js 14+. npm handles PATH auto.",
  },
  {
    id: "source",
    label: "Source",
    code: "git clone https://github.com/BG4JTS/oneinit.git\ncd oneinit && cargo build --release",
    note: "Requires Rust 1.94+. Binary at target/release/oneinit",
  },
];

export function Installation() {
  const [active, setActive] = useState(0);
  const [copied, setCopied] = useState(false);
  const reduce = useReducedMotion();

  const handleCopy = () => {
    navigator.clipboard.writeText(TABS[active].code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section id="install" className="relative border-t border-zinc-800 py-32 md:py-40">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_rgba(16,185,129,0.08)_0%,_transparent_70%)]" />
      <div className="relative z-10 mx-auto max-w-[800px] px-6">
        <h2 className="mb-2 text-center font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">
          Install
        </h2>
        <h3 className="mb-12 text-center text-3xl font-black tracking-tight md:text-5xl">
          One line.{" "}
          <span className="bg-gradient-to-r from-neon to-emerald-400 bg-clip-text text-transparent">
            Done.
          </span>
        </h3>

        <div className="overflow-hidden rounded-2xl border border-zinc-800/60 bg-zinc-900/40 backdrop-blur-md">
          {/* Tabs */}
          <div className="flex border-b border-zinc-800/60">
            {TABS.map((tab, i) => (
              <button
                key={tab.id}
                onClick={() => setActive(i)}
                className={`flex-1 px-4 py-4 font-mono text-sm font-medium transition-all ${
                  active === i
                    ? "bg-zinc-900 text-neon border-b-2 border-neon"
                    : "text-zinc-600 hover:text-zinc-300"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Code */}
          <div className="relative p-6 md:p-8">
            <button
              onClick={handleCopy}
              className="absolute right-6 top-6 rounded-md border border-zinc-700 px-3 py-1.5 font-mono text-xs text-zinc-500 transition-all hover:border-emerald-500/50 hover:text-neon active:scale-95"
            >
              {copied ? "copied" : "copy"}
            </button>
            <pre className="font-mono text-sm leading-relaxed text-zinc-300 md:text-base">
              <code>
                {TABS[active].code.split("\n").map((line, i) => (
                  <div key={i}>
                    <span className="select-none text-neon">$ </span>
                    {line}
                  </div>
                ))}
              </code>
            </pre>
            <p className="mt-4 text-xs text-zinc-600">{TABS[active].note}</p>
          </div>
        </div>
      </div>
    </section>
  );
}
