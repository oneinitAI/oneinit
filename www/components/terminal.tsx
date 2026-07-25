"use client";

import { useState, useEffect, useRef } from "react";
import { motion, useReducedMotion } from "motion/react";

const LINES = [
  { text: "oneinit install python3.11", prompt: true, delay: 0 },
  { text: "", delay: 400 },
  { text: "[OK] Download complete: 10.7 MB", color: "text-zinc-500", delay: 100 },
  { text: "[OK] SHA256 verified", color: "text-zinc-500", delay: 100 },
  { text: "[OK] Extraction complete", color: "text-zinc-500", delay: 100 },
  { text: "[OK] Mirror configured (Tsinghua)", color: "text-emerald-500", delay: 100 },
  { text: "[OK] PATH updated", color: "text-zinc-500", delay: 100 },
  { text: "[SECURITY] Recorded in manifest", color: "text-zinc-600", delay: 100 },
  { text: "", delay: 200 },
  { text: "python --version", prompt: true, delay: 300 },
  { text: "Python 3.11.9", color: "text-emerald-400", delay: 200 },
  { text: "", delay: 100 },
  { text: "pip config get index-url", prompt: true, delay: 300 },
  { text: "https://pypi.tuna.tsinghua.edu.cn/simple", color: "text-emerald-400", delay: 200 },
];

export function Terminal() {
  const reduce = useReducedMotion();
  const [visibleLines, setVisibleLines] = useState<number>(0);
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
        // Restart after pause
        timer = setTimeout(() => {
          setVisibleLines(0);
          currentLine = 0;
          showNext();
        }, 4000);
        return;
      }

      setVisibleLines(currentLine + 1);
      currentLine++;
      timer = setTimeout(showNext, LINES[currentLine - 1]?.delay ?? 200);
    };

    timer = setTimeout(showNext, 800);
    return () => clearTimeout(timer);
  }, [reduce]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [visibleLines]);

  return (
    <div className="w-full max-w-[440px] rounded-xl border border-zinc-800 bg-zinc-900/80 shadow-2xl shadow-emerald-500/5 backdrop-blur-sm overflow-hidden">
      {/* Title bar */}
      <div className="flex items-center gap-2 border-b border-zinc-800 px-4 py-2.5">
        <span className="h-3 w-3 rounded-full bg-zinc-700" />
        <span className="h-3 w-3 rounded-full bg-zinc-700" />
        <span className="h-3 w-3 rounded-full bg-zinc-700" />
        <span className="ml-2 font-mono text-xs text-zinc-600">oneinit - zsh</span>
      </div>

      {/* Terminal body */}
      <div
        ref={scrollRef}
        className="terminal-scroll h-[340px] overflow-y-auto p-4 font-mono text-[13px] leading-relaxed"
      >
        {LINES.slice(0, visibleLines).map((line, i) => (
          <motion.div
            key={i}
            initial={reduce ? undefined : { opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.15 }}
            className={line.color ?? "text-zinc-300"}
          >
            {line.prompt && <span className="text-emerald-500">$ </span>}
            {line.text}
            {i === visibleLines - 1 && visibleLines < LINES.length && (
              <span className="cursor-blink ml-0.5 inline-block h-3.5 w-1.5 bg-emerald-500 align-middle" />
            )}
          </motion.div>
        ))}
      </div>
    </div>
  );
}
