"use client";
import { useLang } from "./lang-provider";

const STEPS = [
  { n: "01", tk: "hiw.s1t", dk: "hiw.s1d" },
  { n: "02", tk: "hiw.s2t", dk: "hiw.s2d" },
  { n: "03", tk: "hiw.s3t", dk: "hiw.s3d" },
  { n: "04", tk: "hiw.s4t", dk: "hiw.s4d" },
];

export function HowItWorks() {
  const { t } = useLang();
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">{t("hiw.badge")}</span>
        <h2 className="mt-3 mb-16 text-3xl font-bold text-white md:text-5xl">{t("hiw.title1")} <span className="text-zinc-600">{t("hiw.title2")}</span></h2>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-4">
          {STEPS.map((s, i) => (
            <div key={i} data-aos="fade-up" data-aos-delay={i * 120}
              className="glass rounded-2xl p-6 transition-all glass-hover hover:-translate-y-1 group">
              <div className="mb-4 font-mono text-4xl font-bold text-zinc-800 group-hover:text-zinc-700 transition-colors">{s.n}</div>
              <h3 className="mb-2 text-lg font-bold text-white">{t(s.tk)}</h3>
              <p className="text-sm leading-relaxed text-zinc-400">{t(s.dk)}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
