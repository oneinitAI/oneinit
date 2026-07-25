"use client";

const ITEMS = [
  { t: "Auto Mirror Config", d: "pip uses Tsinghua. npm uses npmmirror. No config files. No searching for registry URLs. Works out of the box on every install.", i: "globe" },
  { t: "7 Language Detectors", d: "Scan any machine for Python, Node.js, Git, Rust, Go, Java, Docker. Export the blueprint. Import on another machine.", i: "scan" },
  { t: "Community Recipe Registry", d: "Publish YAML recipes. Others install with one command. Like npm, but for dev tools. Versioned, reviewed, secure.", i: "package" },
  { t: "Full Environment Migration", d: "Export your entire setup as a portable tar.gz. Import on a new machine. Tools, configs, package lists — everything restored.", i: "migrate" },
];

export function Features() {
  return (
    <section className="border-t border-[rgba(255,255,255,0.04)] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          {ITEMS.map((item, i) => (
            <div key={i} data-aos="fade-up" data-aos-delay={i * 150}
              className="glass rounded-2xl p-8 transition-all hover:border-cyan/20 hover:shadow-lg hover:shadow-cyan/5 hover:-translate-y-1 group">
              <div className="mb-4 h-10 w-10 rounded-xl bg-gradient-to-br from-cyan/20 to-purple/20 flex items-center justify-center font-mono text-lg">{(i + 1).toString().padStart(2, "0")}</div>
              <h3 className="mb-2 text-xl font-bold text-white">{item.t}</h3>
              <p className="leading-relaxed text-zinc-400">{item.d}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
