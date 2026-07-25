"use client";

import { useState, useEffect, useRef } from "react";
import { motion, useReducedMotion } from "motion/react";

const LINES = [
  { text: "$ oneinit install python3.11", prompt: true, delay: 0 },
  { text: "", delay: 300 },
  { text: "[OK] Download complete (10.7 MB)", color: "text-zinc-500", delay: 80 },
  { text: "[OK] SHA256 verified", color: "text-zinc-500", delay: 80 },
  { text: "[OK] Extraction complete (34 files)", color: "text-zinc-500", delay: 80 },
  { text: "[OK] get-pip bootstrap done", color: "text-zinc-500", delay: 80 },
  { text: "[OK] Mirror: Tsinghua pip configured", color: "text-emerald-500", delay: 120 },
  { text: "[OK] Added to PATH", color: "text-zinc-500", delay: 80 },
  { text: "[OK] Recorded in SQLite manifest", color: "text-zinc-600", delay: 80 },
  { text: "", delay: 300 },
  { text: "$ python --version", prompt: true, delay: 400 },
  { text: "Python 3.11.9", color: "text-emerald-400 font-bold", delay: 250 },
  { text: "$ pip config get index-url", prompt: true, delay: 400 },
  { text: "https://pypi.tuna.tsinghua.edu.cn/simple", color: "text-emerald-400", delay: 250 },
];

export function Terminal() {
  const reduce = useReducedMotion();
  const [visibleLines, setVisibleLines] = useState<number>(0);
  const [showCursor, setShowCursor] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (reduce) {
      setVisibleLines(LINES.length);
      return;
    }

    let currentLine = 0;
    let timer: ReturnType<typeof setTimeout>;

    const showNext = () => {
      if (currentLine >= LINES.length) {
        setShowCursor(false);
        timer = setTimeout(() => {
          setVisibleLines(0);
          setShowCursor(true);
          currentLine = 0;
          showNext();
        }, 5000);
        return;
      }

      setVisibleLines(currentLine + 1);
      currentLine++;
      timer = setTimeout(showNext, LINES[currentLine - 1]?.delay ?? 120);
    };

    timer = setTimeout(showNext, 1000);
    return () => clearTimeout(timer);
  }, [reduce]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [visibleLines]);

  return (
    <div className="group w-full max-w-[460px] rounded-xl border border-zinc-700/50 bg-zinc-950/90 shadow-2xl shadow-emerald-500/10 backdrop-blur-sm overflow-hidden transition-all duration-500 hover:border-zinc-600/80 hover:shadow-emerald-500/20">
      {/* Title bar */}
      <div className="flex items-center gap-2 border-b border-zinc-800/60 px-4 py-2.5">
        <span className="h-3 w-3 rounded-full bg-rose-500/80 transition-colors group-hover:bg-rose-400" />
        <span className="h-3 w-3 rounded-full bg-amber-500/80 transition-colors group-hover:bg-amber-400" />
        <span className="h-3 w-3 rounded-full bg-emerald-500/80 transition-colors group-hover:bg-emerald-400" />
        <span className="ml-2 font-mono text-[11px] text-zinc-600 transition-colors group-hover:text-zinc-500">
          zsh - oneinit
        </span>
      </div>

      {/* Terminal body */}
      <div
        ref={scrollRef}
        className="terminal-scroll h-[360px] overflow-y-auto p-4 font-mono text-[13px] leading-relaxed"
      >
        {LINES.slice(0, visibleLines).map((line, i) => (
          <motion.div
            key={i}
            initial={reduce ? undefined : { opacity: 0, x: -4 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.12 }}
            className={line.color ?? "text-zinc-300"}
          >
            {line.prompt && <span className="text-emerald-500 select-none">$ </span>}
            {line.text}
            {i === visibleLines - 1 && visibleLines < LINES.length && showCursor && (
              <span className="cursor-blink ml-0.5 inline-block h-4 w-2 bg-emerald-500 align-middle" />
            )}
          </motion.div>
        ))}

        {/* Glitch overlay line on hover */}
        <div className="pointer-events-none absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-100">
          <div
            className="absolute left-0 w-full bg-emerald-500/10"
            style={{ height: "2px", top: "60%" }}
          />
        </div>
      </div>
    </div>
  );
}
