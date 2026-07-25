"use client";

import { useState } from "react";
import { motion, useReducedMotion, AnimatePresence } from "motion/react";

const TABS = [
  {
    id: "shell",
    label: "Shell script",
    code: "curl -fsSL https://raw.githubusercontent.com/BG4JTS/oneinit/main/install.sh | sh",
    note: "No prerequisites. Auto-detects OS and architecture.",
  },
  {
    id: "npm",
    label: "npm",
    code: "npm install -g oneinit",
    note: "Requires Node.js 14+. npm handles PATH automatically.",
  },
  {
    id: "source",
    label: "Source",
    code: "git clone https://github.com/BG4JTS/oneinit.git\ncd oneinit && cargo build --release",
    note: "Requires Rust 1.94+. Binary at target/release/oneinit",
  },
];

export function Installation() {
  const [activeTab, setActiveTab] = useState(0);
  const [copied, setCopied] = useState(false);
  const reduce = useReducedMotion();

  const handleCopy = () => {
    navigator.clipboard.writeText(TABS[activeTab].code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section
      id="install"
      className="border-t border-zinc-900 py-24 md:py-32"
    >
      <div className="mx-auto max-w-[800px] px-6">
        <motion.h2
          initial={reduce ? undefined : { opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.5 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          className="mb-3 text-center text-3xl font-bold tracking-tight md:text-4xl"
        >
          Install in seconds
        </motion.h2>
        <p className="mb-10 text-center text-zinc-400">
          Pick whichever works for you. All methods install the same binary.
        </p>

        {/* Tab switcher */}
        <div className="mb-4 flex justify-center gap-1 rounded-xl border border-zinc-800 bg-zinc-900/50 p-1">
          {TABS.map((tab, i) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(i)}
              className={`rounded-lg px-4 py-2 text-sm font-medium transition-all active:scale-[0.97] ${
                activeTab === i
                  ? "bg-emerald-500 text-zinc-950"
                  : "text-zinc-400 hover:text-zinc-100"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Code block */}
        <div className="relative overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900/80">
          <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-2.5">
            <span className="font-mono text-xs text-zinc-600">
              {TABS[activeTab].label}
            </span>
            <button
              onClick={handleCopy}
              className="flex items-center gap-1.5 rounded-md px-2 py-1 font-mono text-xs text-zinc-500 transition-colors hover:bg-zinc-800 hover:text-zinc-300 active:scale-95"
            >
              {copied ? (
                <>
                  <span className="text-emerald-500">copied</span>
                </>
              ) : (
                "copy"
              )}
            </button>
          </div>
          <AnimatePresence mode="wait">
            <motion.pre
              key={activeTab}
              initial={reduce ? undefined : { opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={reduce ? undefined : { opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="terminal-scroll overflow-x-auto p-4 font-mono text-[13px] leading-relaxed text-zinc-300"
            >
              <code>
                {TABS[activeTab].code.split("\n").map((line, i) => (
                  <div key={i}>
                    <span className="text-emerald-500">$ </span>
                    {line}
                  </div>
                ))}
              </code>
            </motion.pre>
          </AnimatePresence>
        </div>

        <p className="mt-4 text-center text-sm text-zinc-500">
          {TABS[activeTab].note}
        </p>
      </div>
    </section>
  );
}
