"use client";

import { useEffect, useRef, useState } from "react";
import { motion, useReducedMotion } from "motion/react";

const STATS = [
  { value: 17, label: "CLI commands", suffix: "" },
  { value: 7, label: "Language detectors", suffix: "" },
  { value: 26, label: "Unit tests pass", suffix: "" },
  { value: 7.3, label: "MB binary size", suffix: "MB" },
];

function CountUp({
  target,
  suffix,
  triggered,
}: {
  target: number;
  suffix: string;
  triggered: boolean;
}) {
  const [value, setValue] = useState(0);
  const reduce = useReducedMotion();

  useEffect(() => {
    if (!triggered || reduce) {
      setValue(target);
      return;
    }

    let start = 0;
    const duration = 1200;
    const step = 16;
    const steps = duration / step;
    const increment = target / steps;
    let current = 0;
    let frame: number;

    function easeOutExpo(t: number) {
      return t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
    }

    function tick() {
      current++;
      const progress = easeOutExpo(current / steps);
      const val = target * progress;
      setValue(val);
      if (current < steps) {
        frame = requestAnimationFrame(tick);
      } else {
        setValue(target);
      }
    }

    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [triggered, target, reduce]);

  const display = target >= 10
    ? Math.round(value).toString()
    : value.toFixed(1);

  return (
    <span className="font-mono text-3xl font-bold tracking-tight text-zinc-100 md:text-4xl">
      {display}{suffix}
    </span>
  );
}

export function Stats() {
  const [triggered, setTriggered] = useState(false);
  const sectionRef = useRef<HTMLDivElement>(null);
  const reduce = useReducedMotion();

  useEffect(() => {
    if (reduce) { setTriggered(true); return; }
    const observer = new IntersectionObserver(
      ([entry]) => { if (entry.isIntersecting) { setTriggered(true); observer.disconnect(); } },
      { threshold: 0.3 }
    );
    if (sectionRef.current) observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, [reduce]);

  return (
    <section ref={sectionRef} className="border-t border-zinc-900 py-20">
      <div className="mx-auto max-w-[1000px] px-6">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          {STATS.map((stat, i) => (
            <motion.div
              key={i}
              initial={reduce ? undefined : { opacity: 0, y: 20 }}
              animate={triggered ? { opacity: 1, y: 0 } : {}}
              transition={{ duration: 0.5, delay: i * 0.12, ease: [0.16, 1, 0.3, 1] }}
              className="text-center"
            >
              <CountUp target={stat.value} suffix={stat.suffix} triggered={triggered} />
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
