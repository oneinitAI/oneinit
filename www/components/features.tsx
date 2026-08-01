"use client";
import { useLang } from "./lang-provider";

const I = [
  { t: "feat.c1t", d: "feat.c1d", n: "01" },
  { t: "feat.c2t", d: "feat.c2d", n: "02" },
  { t: "feat.c3t", d: "feat.c3d", n: "03" },
  { t: "feat.c4t", d: "feat.c4d", n: "04" },
];

export function Features() {
  const { t } = useLang();
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          {I.map((item, i) => (
            <div key={i} data-aos="fade-up" data-aos-delay={i * 150} className="glass rounded-2xl p-8 transition-all glass-hover hover:-translate-y-1 group">
              <div className="mb-4 font-mono text-sm text-emerald-600">{item.n}</div>
              <h3 className="mb-2 text-xl font-bold text-white">{t(item.t)}</h3>
              <p className="leading-relaxed text-zinc-400">{t(item.d)}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
