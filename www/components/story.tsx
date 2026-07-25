"use client";
import { useState } from "react";

const STEPS = [
  { id: "find", q: "Which Python version do I need?", a: "Don't care. Just install." },
  { id: "dl", q: "Go to python.org → Downloads → find the right installer", a: "Auto-detect OS + download." },
  { id: "run", q: "Run .exe → check 'Add to PATH' → Next × 5", a: "Handled. No checkboxes." },
  { id: "pip", q: "Python works. Now how do I install pip?", a: "Bundled. get-pip auto-bootstrap." },
  { id: "mirror", q: "pip is slow. Google 'pip mirror' → edit pip.ini", a: "Tsinghua mirror auto-configured." },
  { id: "verify", q: "Did it actually work? Let me google how to check...", a: "Verified. SHA256, PATH, manifest all confirmed." },
  { id: "remove", q: "I messed up -- how do I uninstall cleanly?", a: "oneinit uninstall. 100% rollback." },
];

const cmds = ["install python3.11", "install node20", "capture", "doctor", "search"];

export function Story() {
  const [expanded, setExpanded] = useState<Record<string,boolean>>({});
  const [activeCmd, setActiveCmd] = useState(0);
  const [typed, setTyped] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const toggle = (id:string) => setExpanded(p => ({...p, [id]: !p[id]}));

  const submit = () => {
    if (!typed.trim()) return;
    setSubmitted(true);
    setTimeout(() => { setSubmitted(false); setTyped(""); }, 2500);
  };

  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">

        {/* Headline */}
        <div className="mb-16 text-center" data-aos="fade-up">
          <span className="font-mono text-xs uppercase tracking-[0.3em] text-rose-500">The Problem</span>
          <h2 className="mt-3 text-3xl font-bold text-white md:text-5xl lg:text-6xl">
            Installing Python<br />
            <span className="text-zinc-600">should not take 30 minutes.</span>
          </h2>
        </div>

        {/* Two columns: pain points vs oneinit */}
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
          {/* Left: Without OneInit */}
          <div data-aos="fade-right">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-rose-500/20 bg-rose-500/5 px-3 py-1">
              <span className="h-1.5 w-1.5 rounded-full bg-rose-500" />
              <span className="font-mono text-xs tracking-wider text-rose-400">Without OneInit</span>
            </div>
            <h3 className="mb-6 text-xl font-bold text-white">The Developer's Afternoon</h3>

            <div className="space-y-2">
              {STEPS.slice(0, 5).map((s, i) => (
                <div key={s.id}
                  onClick={() => toggle(s.id)}
                  data-aos="fade-right" data-aos-delay={i * 80}
                  className="cursor-pointer rounded-xl border border-rose-500/10 bg-rose-500/[0.02] p-4 transition-all hover:border-rose-500/20 hover:bg-rose-500/[0.04]">
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-xs text-rose-500/60">{String(i+1).padStart(2,"0")}</span>
                    <span className="text-sm text-zinc-300">{s.q}</span>
                    <span className="ml-auto text-zinc-700 text-xs">{expanded[s.id] ? "−" : "+"}</span>
                  </div>
                  {expanded[s.id] && (
                    <div className="mt-3 ml-8 border-l border-rose-500/20 pl-4 text-sm text-zinc-500 leading-relaxed">
                      Time lost: ~{i * 4 + 3} minutes. Then you realize you forgot to configure the mirror...
                    </div>
                  )}
                </div>
              ))}
            </div>

            <div className="mt-4 text-center font-mono text-xs text-rose-500/40">~30 minutes. Every machine. Every time.</div>
          </div>

          {/* Right: With OneInit */}
          <div data-aos="fade-left">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-3 py-1">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
              <span className="font-mono text-xs tracking-wider text-emerald-500">With OneInit</span>
            </div>
            <h3 className="mb-6 text-xl font-bold text-white">One Command</h3>

            {/* Interactive terminal input */}
            <div className="overflow-hidden rounded-2xl border border-emerald-500/10 bg-zinc-900/80">
              <div className="flex items-center gap-2 border-b border-white/[0.04] px-4 py-2.5">
                <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
                <span className="h-2.5 w-2.5 rounded-full bg-emerald-600" />
                <span className="ml-2 font-mono text-[11px] text-zinc-600">terminal</span>
              </div>
              <div className="p-4">
                <div className="font-mono text-sm">
                  <span className="text-emerald-500 select-none">$ </span>
                  <span className="text-zinc-300">oneinit </span>
                  <input
                    type="text"
                    value={typed}
                    onChange={e => setTyped(e.target.value)}
                    onKeyDown={e => e.key === "Enter" && submit()}
                    placeholder="install python3.11"
                    className="bg-transparent border-none outline-none text-emerald-400 placeholder:text-zinc-600 font-mono text-sm w-[200px]"
                    spellCheck={false}
                    autoComplete="off"
                  />
                  <span className="inline-block h-4 w-2 bg-emerald-500 animate-pulse align-middle" />
                </div>

                {/* Command pills */}
                <div className="mt-3 flex flex-wrap gap-2">
                  {cmds.map((cmd, i) => (
                    <button key={i}
                      onClick={() => { setActiveCmd(i); setTyped(cmd); }}
                      className={`rounded-full px-3 py-1 font-mono text-xs transition-all ${
                        activeCmd === i
                          ? "bg-emerald-600/20 text-emerald-500 border border-emerald-600/30"
                          : "text-zinc-600 border border-zinc-800 hover:border-zinc-600 hover:text-zinc-400"
                      }`}
                    >
                      {cmd}
                    </button>
                  ))}
                </div>

                {/* Submission result */}
                {submitted && (
                  <div className="mt-4 border-t border-emerald-500/10 pt-4 font-mono text-xs space-y-1 animate-pulse">
                    <div className="text-zinc-500">[OK] {typed} — running...</div>
                    <div className="text-emerald-500">[OK] Done. Your machine is developer-ready.</div>
                  </div>
                )}
              </div>
            </div>

            {/* Summary */}
            <div className="mt-4 space-y-2">
              {STEPS.slice(0, 5).map((s, i) => (
                <div key={s.id}
                  className="flex items-center gap-3 rounded-lg border border-emerald-500/5 bg-emerald-500/[0.01] p-3 transition-all hover:border-emerald-500/15">
                  <span className="flex h-5 w-5 items-center justify-center rounded bg-emerald-600/20 font-mono text-[10px] text-emerald-500">✓</span>
                  <span className="text-xs text-zinc-500">{s.a}</span>
                </div>
              ))}
            </div>

            <div className="mt-4 text-center font-mono text-xs text-emerald-500/60">One command. Less than 30 seconds. Every machine.</div>
          </div>
        </div>
      </div>
    </section>
  );
}
