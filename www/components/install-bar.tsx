"use client";
import { useState } from "react";

const TABS = [
  { id: "shell", l: "Shell", c: "curl -fsSL https://raw.githubusercontent.com/BG4JTS/oneinit/main/install.sh | sh", n: "Zero prerequisites. Auto-detects OS." },
  { id: "npm", l: "npm", c: "npm install -g oneinit", n: "Node.js 14+. PATH handled automatically." },
  { id: "source", l: "Source", c: "git clone https://github.com/BG4JTS/oneinit.git\ncd oneinit && cargo build --release", n: "Rust 1.94+. Binary at target/release." },
];

export function InstallBar() {
  const [active, setActive] = useState(0);
  const [copied, setCopied] = useState(false);

  const copy = () => { navigator.clipboard.writeText(TABS[active].c); setCopied(true); setTimeout(() => setCopied(false), 2000); };

  return (
    <section id="install" className="border-t border-[rgba(255,255,255,0.04)] py-24 md:py-32" data-aos="fade-up">
      <div className="mx-auto max-w-[750px] px-6 text-center">
        <span className="font-mono text-xs uppercase tracking-[0.3em] text-cyan">Get Started</span>
        <h2 className="mt-3 mb-2 text-3xl font-bold md:text-5xl">Install in <span className="text-gradient">seconds</span>.</h2>
        <p className="mb-10 text-zinc-500">Pick your method. All install the same binary.</p>

        <div className="glass overflow-hidden rounded-2xl">
          <div className="flex border-b border-[rgba(255,255,255,0.04)]">
            {TABS.map((tab, i) => (
              <button key={tab.id} onClick={() => setActive(i)}
                className={`flex-1 py-3 font-mono text-sm transition-all ${active === i ? "text-cyan border-b-2 border-cyan bg-[rgba(0,240,255,0.03)]" : "text-zinc-600 hover:text-zinc-300"}`}>
                {tab.l}
              </button>
            ))}
          </div>
          <div className="relative p-6 text-left">
            <button onClick={copy} className="absolute right-4 top-4 glass-hover rounded-lg px-3 py-1.5 font-mono text-xs text-zinc-400 transition-all">
              {copied ? "copied!" : "copy"}
            </button>
            <pre className="font-mono text-sm leading-relaxed text-zinc-200"><code>
              {TABS[active].c.split("\n").map((l, i) => <div key={i}><span className="select-none text-cyan">$ </span>{l}</div>)}
            </code></pre>
            <p className="mt-3 text-xs text-zinc-600">{TABS[active].n}</p>
          </div>
        </div>
      </div>
    </section>
  );
}
