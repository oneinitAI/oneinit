"use client";
const ROWS = [
  ["Download installer, Next, Next, Finish", "oneinit install python3.11"],
  ["Manually configure pip / npm mirrors", "Auto: Tsinghua / npmmirror"],
  ["rm -rf and hope it's clean", "SQLite manifest, 100% rollback"],
  ["\"What did I install before?\"", "oneinit capture, oneinit export"],
  ["New machine = lost afternoon", "oneinit import backup.tar.gz"],
];
export function Comparison() {
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1000px] px-6">
        <span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">Comparison</span>
        <h2 className="mt-3 mb-12 text-3xl font-bold text-white md:text-5xl">New machine = hours of setup. <span className="text-zinc-600">Until now.</span></h2>
        <div className="hidden md:grid grid-cols-[1fr_auto_1fr] gap-4 mb-2 px-6">
          <span className="font-mono text-xs uppercase tracking-wider text-zinc-600">Traditional</span>
          <span />
          <span className="font-mono text-xs uppercase tracking-wider text-emerald-500">OneInit</span>
        </div>
        <div className="space-y-1">
          {ROWS.map((r,i) => (
            <div key={i} data-aos="fade-right" data-aos-delay={i*80}
              className="grid grid-cols-[1fr_auto_1fr] items-center gap-3 rounded-xl glass p-4 md:p-5 glass-hover transition-all">
              <div className="text-right text-sm text-zinc-500 line-through decoration-zinc-700/50 md:text-base">{r[0]}</div>
              <div className="font-mono text-zinc-700 text-lg px-2">&rarr;</div>
              <div className="font-mono text-sm text-zinc-100 md:text-base"><span className="text-emerald-500">$ </span>{r[1]}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
