"use client";
import { useState } from "react";
import { useLang } from "./lang-provider";

const T = [
  { id: "shell", lk: "ib.shell", c: "curl -fsSL https://raw.githubusercontent.com/oneinitAI/oneinit/main/install.sh | sh", nk: "ib.shell_n" },
  { id: "npm", lk: "ib.npm", c: "npm install -g oneinit", nk: "ib.npm_n" },
  { id: "source", lk: "ib.source", c: "git clone https://github.com/oneinitAI/oneinit.git\ncd oneinit && cargo build --release", nk: "ib.source_n" },
];

export function InstallBar() {
  const { t } = useLang();
  const [a, setA] = useState(0);
  const [c, setC] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(T[a].c);
    setC(true);
    setTimeout(() => setC(false), 2000);
  };
  return (
    <section id="install" className="border-t border-white/[0.04] py-24 md:py-32" data-aos="fade-up">
      <div className="mx-auto max-w-[750px] px-6 text-center">
        <span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">{t("ib.badge")}</span>
        <h2 className="mt-3 mb-2 text-3xl font-bold text-white md:text-5xl">{t("ib.title1")} <span className="text-zinc-600">{t("ib.title2")}</span></h2>
        <p className="mb-10 text-zinc-500">{t("ib.desc")}</p>
        <div className="glass overflow-hidden rounded-2xl">
          <div className="flex border-b border-white/[0.04]">
            {T.map((tab, i) => (
              <button key={tab.id} onClick={() => setA(i)}
                className={`flex-1 py-3 font-mono text-sm transition-all ${a === i ? "text-emerald-600 border-b-2 border-emerald-600 bg-emerald-600/[0.02]" : "text-zinc-600 hover:text-zinc-300"}`}>
                {t(tab.lk)}
              </button>
            ))}
          </div>
          <div className="relative p-6 text-left">
            <button onClick={copy}
              className="absolute right-4 top-4 glass-hover rounded-lg px-3 py-1.5 font-mono text-xs text-zinc-400 transition-all">
              {c ? t("ib.copied") : t("ib.copy")}
            </button>
            <pre className="font-mono text-sm leading-relaxed text-zinc-200">
              <code>{T[a].c.split("\n").map((l, i) => (
                <div key={i}><span className="select-none text-emerald-600">$ </span>{l}</div>
              ))}</code>
            </pre>
            <p className="mt-3 text-xs text-zinc-600">{t(T[a].nk)}</p>
          </div>
        </div>
      </div>
    </section>
  );
}
