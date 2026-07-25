"use client";

import { useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

const ITEMS = [
  { cmd: "install python3.11", out: "Python 3.11.9", tag: "Python + pip + Tsinghua" },
  { cmd: "install node20", out: "Node.js 20.18.1", tag: "Node + npm + npmmirror" },
  { cmd: "capture -o env.yaml", out: "6 environments", tag: "Python, Node, Git, Rust, Go, Java" },
  { cmd: "export --include-envs", out: "backup.tar.gz", tag: "Full environment backup" },
  { cmd: "import backup.tar.gz", out: "restored", tag: "New machine ready" },
  { cmd: "search", out: "builtin + 0 remote", tag: "3-tier recipe search" },
  { cmd: "publish recipe.yaml", out: "published", tag: "Community registry" },
  { cmd: "doctor", out: "6/6 passed", tag: "Environment healthy" },
];

gsap.registerPlugin(ScrollTrigger);

export function HorizontalScroll() {
  const reduce = useReducedMotion();
  const wrapRef = useRef<HTMLDivElement>(null);
  const trackRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (reduce || !wrapRef.current || !trackRef.current) return;
    const ctx = gsap.context(() => {
      const distance = trackRef.current!.scrollWidth - window.innerWidth;
      if (distance <= 0) return;
      gsap.to(trackRef.current, {
        x: -distance,
        ease: "none",
        scrollTrigger: {
          trigger: wrapRef.current,
          start: "top top",
          end: () => `+=${distance}`,
          pin: true,
          scrub: 1,
          invalidateOnRefresh: true,
        },
      });
    }, wrapRef);
    return () => ctx.revert();
  }, [reduce]);

  return (
    <section
      ref={wrapRef}
      className="relative overflow-hidden border-t border-zinc-800"
    >
      <div className="absolute top-0 left-0 z-20 p-8">
        <div className="font-mono text-xs uppercase tracking-[0.3em] text-neon">
          Try these
        </div>
        <h3 className="mt-2 text-2xl font-black">Command Wall</h3>
        <p className="mt-1 text-sm text-zinc-500">Scroll horizontally</p>
      </div>

      <div ref={trackRef} className="flex h-[100dvh] items-center gap-8 pl-[320px] pr-16">
        {ITEMS.map((item, i) => (
          <div
            key={i}
            className="flex-shrink-0 group relative rounded-2xl border border-zinc-800/60 bg-zinc-900/40 p-8 backdrop-blur-sm transition-all duration-300 hover:border-neon/30 w-[380px]"
          >
            <div className="mb-3 inline-block rounded-full bg-zinc-800 px-3 py-1 font-mono text-[10px] uppercase tracking-wider text-zinc-500">
              {item.tag}
            </div>
            <div className="mb-2 font-mono text-lg text-zinc-100">
              <span className="text-neon">$ </span>
              {item.cmd}
            </div>
            <div className="font-mono text-sm text-emerald-400">
              &rarr; {item.out}
            </div>
            {/* Card number */}
            <div className="absolute -right-3 -top-6 select-none font-mono text-8xl font-black text-zinc-900 transition-colors group-hover:text-zinc-800">
              {String(i + 1).padStart(2, "0")}
            </div>
          </div>
        ))}
        {/* End marker */}
        <div className="flex-shrink-0 flex items-center justify-center w-[300px] h-[200px]">
          <div className="text-center font-mono text-zinc-700">
            <div className="text-6xl mb-4">&infin;</div>
            <div className="text-xs uppercase tracking-widest">More commands</div>
          </div>
        </div>
      </div>
    </section>
  );
}
