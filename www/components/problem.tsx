"use client";

import { motion, useReducedMotion } from "motion/react";

const ROWS = [
  {
    traditional: "Download installer, Next, Next, Finish",
    oneinit: "oneinit install python3.11",
  },
  {
    traditional: "Manually configure pip / npm mirrors",
    oneinit: "Auto: Tsinghua / npmmirror",
  },
  {
    traditional: "rm -rf and hope it's clean",
    oneinit: "SQLite manifest, 100% rollback",
  },
  {
    traditional: '"What did I install before?"',
    oneinit: "oneinit capture, oneinit export",
  },
  {
    traditional: "New machine setup takes an afternoon",
    oneinit: "oneinit import backup.tar.gz",
  },
];

export function Problem() {
  const reduce = useReducedMotion();

  return (
    <section className="border-t border-zinc-900 py-24 md:py-32">
      <div className="mx-auto max-w-[1200px] px-6">
        <motion.h2
          initial={reduce ? undefined : { opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, amount: 0.4 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          className="max-w-[600px] text-3xl font-bold leading-tight tracking-tight md:text-4xl"
        >
          New machine means hours of setup.
          <br />
          <span className="text-zinc-600">Until now.</span>
        </motion.h2>

        <div className="mt-16 grid grid-cols-1 gap-x-12 md:grid-cols-2">
          {/* Traditional */}
          <div className="hidden md:block">
            <div className="mb-6 font-mono text-xs uppercase tracking-wider text-zinc-700">
              Traditional
            </div>
            <div className="divide-y divide-zinc-900">
              {ROWS.map((row, i) => (
                <div key={i} className="py-4 text-sm text-zinc-600 line-through decoration-zinc-800">
                  {row.traditional}
                </div>
              ))}
            </div>
          </div>

          {/* OneInit */}
          <div>
            <div className="mb-6 font-mono text-xs uppercase tracking-wider text-emerald-500">
              OneInit
            </div>
            <div className="divide-y divide-zinc-800">
              {ROWS.map((row, i) => (
                <motion.div
                  key={i}
                  initial={reduce ? undefined : { opacity: 0, x: 20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true, amount: 0.3 }}
                  transition={{ duration: 0.5, delay: i * 0.08, ease: [0.16, 1, 0.3, 1] }}
                  className="py-4 font-mono text-sm text-zinc-100"
                >
                  {row.oneinit}
                </motion.div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
