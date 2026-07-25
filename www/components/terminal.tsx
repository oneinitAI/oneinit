"use client";
import { useState, useEffect, useRef } from "react";

const LINES = [
  { t: "$ oneinit install python3.11", c: "" },
  { t: "[OK] Download complete (10.7 MB)", c: "text-zinc-500" },
  { t: "[OK] SHA256 verified", c: "text-zinc-500" },
  { t: "[OK] Mirror: Tsinghua configured", c: "text-cyan" },
  { t: "[OK] PATH updated", c: "text-zinc-500" },
  { t: "", c: "" },
  { t: "$ python --version", c: "" },
  { t: "Python 3.11.9", c: "text-neon" },
];

export function Terminal() {
  const [vis, setVis] = useState(0);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let i = 0; const t = setInterval(() => { setVis(++i); if (i >= LINES.length) clearInterval(t); }, 350);
    return () => clearInterval(t);
  }, []);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [vis]);

  return (
    <div className="glass w-full max-w-[460px] overflow-hidden rounded-2xl shadow-2xl shadow-purple/5">
      <div className="flex items-center gap-2 border-b border-[rgba(255,255,255,0.04)] px-4 py-2.5">
        <span className="h-3 w-3 rounded-full bg-red-500/80" /><span className="h-3 w-3 rounded-full bg-amber-500/80" /><span className="h-3 w-3 rounded-full bg-neon/80 shadow-[0_0_6px_rgba(0,255,65,0.4)]" />
        <span className="ml-2 font-mono text-[11px] text-zinc-600">Terminal · oneinit</span>
      </div>
      <div ref={scrollRef} className="terminal-scroll h-[340px] overflow-y-auto p-4 font-mono text-[13px] leading-relaxed">
        {LINES.slice(0, vis).map((line, i) => (
          <div key={i} className={line.c || "text-zinc-300"}>
            {!line.c && line.t.startsWith("$") && <span className="text-cyan select-none">$ </span>}
            {line.t}
            {i === vis - 1 && vis < LINES.length && <span className="inline-block h-4 w-2 bg-cyan animate-pulse align-middle ml-0.5" />}
          </div>
        ))}
      </div>
    </div>
  );
}
