"use client";
import { useState } from "react";

const PROMPTS = [
  {
    lang: "English",
    prompt: `Install OneInit on this machine and set up my dev environment.
Follow the guide at:
https://raw.githubusercontent.com/oneinitAI/oneinit/main/INSTALL_AGENT.md`,
  },
  {
    lang: "中文",
    prompt: `在这台机器上安装 OneInit 并配置开发环境。
按照以下指南操作：
https://raw.githubusercontent.com/oneinitAI/oneinit/main/INSTALL_AGENT.md`,
  },
];

export function AIInstall() {
  const [active, setActive] = useState(0);
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(PROMPTS[active].prompt);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32" data-aos="fade-up">
      <div className="mx-auto max-w-[750px] px-6 text-center">
        <span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">
          AI Install
        </span>
        <h2 className="mt-3 mb-2 text-3xl font-bold text-white md:text-5xl">
          Let AI do it.<br />
          <span className="text-zinc-600">Copy. Paste. Done.</span>
        </h2>
        <p className="mb-10 text-zinc-500">
          Don't want to open a terminal? Ask ChatGPT, Claude, or ZCode to install
          OneInit for you. One prompt is all it takes.
        </p>

        <div className="glass overflow-hidden rounded-2xl">
          <div className="flex border-b border-white/[0.04]">
            {PROMPTS.map((p, i) => (
              <button
                key={p.lang}
                onClick={() => setActive(i)}
                className={`flex-1 py-3 font-mono text-sm transition-all ${
                  i === active
                    ? "text-emerald-600 border-b-2 border-emerald-600 bg-emerald-600/[0.02]"
                    : "text-zinc-600 hover:text-zinc-300"
                }`}
              >
                {p.lang}
              </button>
            ))}
          </div>
          <div className="relative p-6 text-left">
            <button
              onClick={copy}
              className="absolute right-4 top-4 glass-hover rounded-lg px-3 py-1.5 font-mono text-xs text-zinc-400 transition-all"
            >
              {copied ? "copied!" : "copy"}
            </button>
            <pre className="font-mono text-sm leading-relaxed text-zinc-200 whitespace-pre-wrap">
              <code>{PROMPTS[active].prompt}</code>
            </pre>
          </div>
        </div>

        <p className="mt-6 text-xs text-zinc-600">
          Works with ChatGPT · Claude · ZCode · Copilot · any AI assistant
        </p>
      </div>
    </section>
  );
}
