"use client";

import { useState, useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";

const SCRIPT = [
  { t: "$ oneinit install python3.11", p: true, c: "", d: 0 },
  { t: "", d: 250 },
  { t: "[OK] Download complete (10.7 MB)", c: "text-zinc-500", d: 60 },
  { t: "[OK] SHA256 verified", c: "text-zinc-500", d: 60 },
  { t: "[OK] get-pip bootstrap done", c: "text-emerald-500", d: 80 },
  { t: "[OK] Mirror: Tsinghua configured", c: "text-neon", d: 100 },
  { t: "[OK] Added to PATH", c: "text-zinc-500", d: 60 },
  { t: "[OK] SQLite manifest recorded", c: "text-zinc-600", d: 60 },
  { t: "", d: 250 },
  { t: "$ python --version", p: true, c: "", d: 350 },
  { t: "Python 3.11.9", c: "text-neon font-bold", d: 200 },
  { t: "$ pip config get index-url", p: true, c: "", d: 350 },
  { t: "https://pypi.tuna.tsinghua.edu.cn/simple", c: "text-neon truncate", d: 200 },
];

export function Terminal() {
  const reduce = useReducedMotion();
  const [vis, setVis] = useState(0);
  const [cursor, setCursor] = useState(true);
  const [glitch, setGlitch] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (reduce) { setVis(SCRIPT.length); return; }
    let i = 0; let t: ReturnType<typeof setTimeout>;
    const next = () => {
      if (i >= SCRIPT.length) { setCursor(false); t = setTimeout(() => { setVis(0); setCursor(true); i = 0; next(); }, 5000); return; }
      setVis(i + 1); i++; t = setTimeout(next, SCRIPT[i - 1]?.d ?? 100);
    };
    t = setTimeout(next, 800);
    return () => clearTimeout(t);
  }, [reduce]);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [vis]);

  // Random glitch trigger
  useEffect(() => {
    const iv = setInterval(() => {
      if (Math.random() > 0.7) { setGlitch(true); setTimeout(() => setGlitch(false), 150); }
    }, 3000);
    return () => clearInterval(iv);
  }, []);

  return (
    <div className="group relative w-full max-w-[480px] rounded-xl border border-zinc-700/40 bg-zinc-950/95 shadow-2xl shadow-neon/10 backdrop-blur-sm overflow-hidden transition-all duration-500 hover:border-neon/30 hover:shadow-neon/20">
      <div className="flex items-center gap-2 border-b border-zinc-800/50 px-4 py-2.5">
        <span className="h-3 w-3 rounded-full bg-rose-500/80" />
        <span className="h-3 w-3 rounded-full bg-amber-500/80" />
        <span className="h-3 w-3 rounded-full bg-neon/80 shadow-[0_0_6px_rgba(0,255,136,0.4)]" />
        <span className="ml-2 font-mono text-[11px] text-zinc-600">zsh · oneinit</span>
      </div>
      <div ref={scrollRef} className={`terminal-scroll h-[370px] overflow-y-auto p-4 font-mono text-[13px] leading-relaxed transition-all duration-100 ${glitch ? "translate-x-[2px] opacity-90" : ""}`}>
        {SCRIPT.slice(0, vis).map((line, i) => (
          <div key={i} className={line.c || "text-zinc-300"}>
            {line.p && <span className="text-neon select-none">$ </span>}
            {line.t}
            {i === vis - 1 && vis < SCRIPT.length && cursor && (
              <span className="cursor-blink ml-0.5 inline-block h-4 w-2 bg-neon align-middle shadow-[0_0_6px_rgba(0,255,136,0.6)]" />
            )}
          </div>
        ))}
      </div>
      {/* Glitch overlay */}
      <div className="pointer-events-none absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-100">
        <div className="absolute left-0 h-[1px] w-full bg-neon/20" style={{ top: "35%" }} />
        <div className="absolute left-0 h-[1px] w-full bg-crimson/20" style={{ top: "68%" }} />
      </div>
    </div>
  );
}
