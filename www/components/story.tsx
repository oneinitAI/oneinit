"use client";
import { useState } from "react";
import { useLang } from "./lang-provider";

const STEPS = [
  { id: "find", qk: "story.q1", ak: "story.a1" },
  { id: "dl", qk: "story.q2", ak: "story.a2" },
  { id: "run", qk: "story.q3", ak: "story.a3" },
  { id: "pip", qk: "story.q4", ak: "story.a4" },
  { id: "mirror", qk: "story.q5", ak: "story.a5" },
  { id: "verify", qk: "story.q6", ak: "story.a6" },
  { id: "remove", qk: "story.q7", ak: "story.a7" },
];

const cmds = ["install python3.11", "install node20", "capture", "doctor", "search"];

export function Story() {
  const { t } = useLang();
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [activeCmd, setActiveCmd] = useState(0);
  const [typed, setTyped] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const toggle = (id: string) => setExpanded((p) => ({ ...p, [id]: !p[id] }));

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
          <span className="font-mono text-xs uppercase tracking-[0.3em] text-rose-500">{t("story.badge")}</span>
          <h2 className="mt-3 text-3xl font-bold text-white md:text-5xl lg:text-6xl">
            {t("story.title1")}<br />
            <span className="text-zinc-600">{t("story.title2")}</span>
          </h2>
        </div>

        {/* Two columns: pain points vs oneinit */}
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
          {/* Left: Without OneInit */}
          <div data-aos="fade-right">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-rose-500/20 bg-rose-500/5 px-3 py-1">
              <span className="h-1.5 w-1.5 rounded-full bg-rose-500" />
              <span className="font-mono text-xs tracking-wider text-rose-400">{t("story.without")}</span>
            </div>
            <h3 className="mb-6 text-xl font-bold text-white">{t("story.afternoon")}</h3>

            <div className="space-y-2">
              {STEPS.slice(0, 5).map((s, i) => (
                <div key={s.id}
                  onClick={() => toggle(s.id)}
                  data-aos="fade-right" data-aos-delay={i * 80}
                  className="cursor-pointer rounded-xl border border-rose-500/10 bg-rose-500/[0.02] p-4 transition-all hover:border-rose-500/20 hover:bg-rose-500/[0.04]">
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-xs text-rose-500/60">{String(i + 1).padStart(2, "0")}</span>
                    <span className="text-sm text-zinc-300">{t(s.qk)}</span>
                    <span className="ml-auto text-zinc-700 text-xs">{expanded[s.id] ? "−" : "+"}</span>
                  </div>
                  {expanded[s.id] && (
                    <div className="mt-3 ml-8 border-l border-rose-500/20 pl-4 text-sm text-zinc-500 leading-relaxed">
                      {t("story.timelost", { n: i * 4 + 3 })}
                    </div>
                  )}
                </div>
              ))}
            </div>

            <div className="mt-4 text-center font-mono text-xs text-rose-500/40">{t("story.summary1")}</div>
          </div>

          {/* Right: With OneInit */}
          <div data-aos="fade-left">
            <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-3 py-1">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
              <span className="font-mono text-xs tracking-wider text-emerald-500">{t("story.with")}</span>
            </div>
            <h3 className="mb-6 text-xl font-bold text-white">{t("story.onecmd")}</h3>

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
                    onChange={(e) => setTyped(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && submit()}
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
                    <div className="text-zinc-500">{t("story.running", { cmd: typed })}</div>
                    <div className="text-emerald-500">{t("story.done")}</div>
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
                  <span className="text-xs text-zinc-500">{t(s.ak)}</span>
                </div>
              ))}
            </div>

            <div className="mt-4 text-center font-mono text-xs text-emerald-500/60">{t("story.summary2")}</div>
          </div>
        </div>
      </div>
    </section>
  );
}
