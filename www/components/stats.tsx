"use client";

import { motion, useReducedMotion } from "motion/react";

const STATS = [
  { value: "17", label: "CLI commands" },
  { value: "7", label: "Language detectors" },
  { value: "26", label: "Unit tests" },
  { value: "7.3MB", label: "Binary size" },
];

export function Stats() {
  const reduce = useReducedMotion();

  return (
    <section className="border-t border-zinc-900 py-20">
      <div className="mx-auto max-w-[1000px] px-6">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          {STATS.map((stat, i) => (
            <motion.div
              key={i}
              initial={reduce ? undefined : { opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.5 }}
              transition={{ duration: 0.5, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
              className="text-center"
            >
              <div className="font-mono text-3xl font-bold tracking-tight text-zinc-100 md:text-4xl">
                {stat.value}
              </div>
              <div className="mt-1 text-xs uppercase tracking-wider text-zinc-600">
                {stat.label}
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
