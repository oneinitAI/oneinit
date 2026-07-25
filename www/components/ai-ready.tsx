"use client";

const CARDS = [
  {
    n: "01", t: "JSON-First Output",
    d: "Every command supports --json. Structured output that AI agents parse directly. No text scraping needed.",
    code: '{\n  "status": "success",\n  "action": "install",\n  "package": "python3.11",\n  "install_path": "~/.oneinit/envs/python3.11",\n  "duration_ms": 3420\n}',
  },
  {
    n: "02", t: "AI Skill Installer",
    d: "One command installs the OneInit Skill into ZCode, Claude, Codex, and Cursor. AI agents can then install tools, capture environments, and migrate machines autonomously.",
    code: "$ oneinit skill install\n[OK] installed -> zcode/codex/claude/agents\n[OK] Skill installed to 4 AI agents",
  },
  {
    n: "03", t: "Agent Installation Guide",
    d: "Give your AI assistant one prompt. It clones the repo, builds the binary, installs tools, and configures the Skill. Zero human intervention.",
    code: "$ curl -fsSL https://raw.githubusercontent.com/\nBG4JTS/oneinit/main/install.sh | sh\n$ oneinit skill install\n$ oneinit install python3.11 --json\n$ oneinit install node20 --json",
  },
  {
    n: "04", t: "AI Agent Autonomous DevOps",
    d: "AI agents can capture environments, export backups, and restore on new machines. CI/CD pipelines can self-bootstrap with one command. Zero manual config.",
    code: "# Agent-initiated migration:\noneinit capture -o env.yaml\noneinit export --include-envs\noneinit import backup.tar.gz --force",
  },
];

export function AIReady() {
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <div className="text-center mb-16" data-aos="fade-up">
          <span className="inline-flex items-center gap-2 rounded-full border border-emerald-500/20 bg-emerald-500/5 px-4 py-1.5 mb-4">
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
            <span className="font-mono text-xs tracking-widest text-emerald-500">AI-NATIVE</span>
          </span>
          <h2 className="text-3xl font-bold text-white md:text-5xl lg:text-6xl">
            Built for <span className="text-emerald-500">AI agents</span>.
            <br />
            <span className="text-zinc-600">Not just developers.</span>
          </h2>
          <p className="mt-4 max-w-[600px] mx-auto text-zinc-400">
            OneInit is the first environment initializer designed for the AI era. Every command has a JSON mode. Every operation is parseable. AI agents don&apos;t read terminal output — they read structured data.
          </p>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          {CARDS.map((card, i) => (
            <div key={i} data-aos="fade-up" data-aos-delay={i * 120}
              className="glass rounded-2xl overflow-hidden transition-all glass-hover hover:-translate-y-1 group">
              <div className="p-6 md:p-8">
                <div className="mb-3 font-mono text-sm text-emerald-600">{card.n}</div>
                <h3 className="mb-2 text-xl font-bold text-white">{card.t}</h3>
                <p className="text-sm leading-relaxed text-zinc-400 mb-4">{card.d}</p>
              </div>
              <div className="border-t border-white/[0.04] bg-zinc-900/50 p-4 md:p-6">
                <pre className="font-mono text-xs leading-relaxed text-zinc-300 terminal-scroll overflow-x-auto"><code>{card.code}</code></pre>
              </div>
            </div>
          ))}
        </div>

        <div className="mt-12 text-center" data-aos="fade-up" data-aos-delay="400">
          <p className="font-mono text-sm text-zinc-600">
            $ oneinit install python3.11 <span className="text-emerald-500">--json</span> <span className="text-zinc-500">→ AI reads</span> <span className="text-emerald-500">{"{"}"status":"success"{")"}</span>
          </p>
        </div>
      </div>
    </section>
  );
}
