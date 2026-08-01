"use client";

import { motion } from "motion/react";

interface FinalCTAScreenProps {
  onReplay?: () => void;
}

export function FinalCTAScreen({ onReplay }: FinalCTAScreenProps) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.6 }}
      className="flex flex-col items-center justify-center py-10 px-6 text-center"
    >
      {/* Slogan */}
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2, duration: 0.5 }}
      >
        <p className="text-2xl md:text-3xl font-bold text-[#f0f0f0] leading-tight">
          One command.
        </p>
        <p className="text-2xl md:text-3xl font-bold text-[#555] leading-tight mt-1">
          Any machine.
        </p>
        <p className="text-2xl md:text-3xl font-bold bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent leading-tight mt-1">
          Ready to code.
        </p>
      </motion.div>

      {/* Key benefits tags */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5, duration: 0.4 }}
        className="flex flex-wrap justify-center gap-2 mt-6"
      >
        {[
          "Auto PATH config",
          "Mirror auto-setup",
          "SQLite rollback",
          "Env migration",
          "AI agent autonomous",
          "--json output",
        ].map((benefit) => (
          <span
            key={benefit}
            className="px-3 py-1 rounded-full text-[11px] border border-emerald-500/20 bg-emerald-500/5 text-emerald-400"
          >
            {benefit}
          </span>
        ))}
      </motion.div>

      {/* Prominent URL cards */}
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.8, duration: 0.5 }}
        className="flex flex-col sm:flex-row gap-4 mt-8 w-full max-w-lg"
      >
        {/* Website card */}
        <a
          href="https://oneinit.bg4jts.cn"
          target="_blank"
          rel="noopener noreferrer"
          className="flex-1 group flex items-center gap-3 rounded-xl border border-emerald-500/30 bg-emerald-500/5 px-5 py-4 transition-all hover:bg-emerald-500/10 hover:border-emerald-500/50 hover:-translate-y-0.5"
        >
          <div className="w-9 h-9 rounded-lg bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center shrink-0 group-hover:bg-emerald-500/25 transition-colors">
            <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
            </svg>
          </div>
          <div className="text-left min-w-0">
            <div className="text-[10px] uppercase tracking-widest text-[#555] mb-0.5">Website</div>
            <div className="text-sm font-mono text-[#d4d4d4] group-hover:text-white truncate transition-colors">
              oneinit.bg4jts.cn
            </div>
          </div>
          <svg className="w-3.5 h-3.5 text-[#555] group-hover:text-emerald-400 shrink-0 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
        </a>

        {/* GitHub card */}
        <a
          href="https://github.com/oneinitAI/oneinit"
          target="_blank"
          rel="noopener noreferrer"
          className="flex-1 group flex items-center gap-3 rounded-xl border border-[#2a2a2a] bg-[#141414] px-5 py-4 transition-all hover:bg-[#1c1c1c] hover:border-[#444] hover:-translate-y-0.5"
        >
          <div className="w-9 h-9 rounded-lg bg-[#1c1c1c] border border-[#333] flex items-center justify-center shrink-0 group-hover:bg-[#2a2a2a] transition-colors">
            <svg className="w-4 h-4 text-[#ccc]" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
            </svg>
          </div>
          <div className="text-left min-w-0">
            <div className="text-[10px] uppercase tracking-widest text-[#555] mb-0.5">GitHub</div>
            <div className="text-sm font-mono text-[#d4d4d4] group-hover:text-white truncate transition-colors">
              oneinitAI/oneinit
            </div>
          </div>
          <svg className="w-3.5 h-3.5 text-[#555] group-hover:text-[#ccc] shrink-0 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
        </a>
      </motion.div>

      {/* Replay */}
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.3, duration: 0.4 }}
        className="mt-5"
      >
        <button
          onClick={onReplay}
          className="bg-transparent border-none text-[#444] text-xs cursor-pointer hover:text-emerald-500 transition-colors font-mono"
        >
          Replay demo
        </button>
      </motion.p>
    </motion.div>
  );
}
