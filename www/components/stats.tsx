"use client";

const DATA = [
  { v: "17", l: "CLI Commands" },
  { v: "7", l: "Language Detectors" },
  { v: "26", l: "Unit Tests Pass" },
  { v: "7.3MB", l: "Binary Size" },
];

export function Stats() {
  return (
    <section className="border-t border-[rgba(255,255,255,0.04)] py-20">
      <div className="mx-auto max-w-[1000px] px-6">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          {DATA.map((d, i) => (
            <div key={i} data-aos="fade-up" data-aos-delay={i * 100} className="text-center glass rounded-2xl p-6 transition-all hover:border-cyan/20 hover:-translate-y-1">
              <div className="font-mono text-3xl font-bold text-white md:text-4xl">{d.v}</div>
              <div className="mt-2 font-mono text-xs uppercase tracking-widest text-zinc-500">{d.l}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
