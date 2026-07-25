"use client";
const STEPS = [
  { n:"01", t:"Install", d:"Write a YAML recipe with the download URL, SHA256, and install steps. Drop it in ~/.oneinit/recipes/." },
  { n:"02", t:"Verify", d:"OneInit downloads the archive, verifies the SHA256 checksum, and extracts it to a sandboxed directory." },
  { n:"03", t:"Configure", d:"Mirror sources are auto-configured. PATH entries are added. Config files are written. All tracked in SQLite." },
  { n:"04", t:"Rollback", d:"Uninstall removes everything: PATH entries, config files, install directory, and manifest record. Complete." },
];
export function HowItWorks() {
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">How It Works</span>
        <h2 className="mt-3 mb-16 text-3xl font-bold text-white md:text-5xl">Install, configure, rollback. <span className="text-zinc-600">Clean.</span></h2>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-4">
          {STEPS.map((s,i) => (
            <div key={i} data-aos="fade-up" data-aos-delay={i*120}
              className="glass rounded-2xl p-6 transition-all glass-hover hover:-translate-y-1 group">
              <div className="mb-4 font-mono text-4xl font-bold text-zinc-800 group-hover:text-zinc-700 transition-colors">{s.n}</div>
              <h3 className="mb-2 text-lg font-bold text-white">{s.t}</h3>
              <p className="text-sm leading-relaxed text-zinc-400">{s.d}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
