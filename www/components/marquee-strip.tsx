export function MarqueeStrip() {
  const items = [
    "Rust Edition 2024", "Zero runtime", "26 unit tests", "7 language detectors",
    "SQLite WAL mode", "100% rollback", "Tsinghua mirror auto-config",
    "SHA256 verified", "Async download engine", "Cross-platform PATH",
    "TUI interactive menu", "npm-like registry", "AI Skill auto-install",
    "Shell completions", "Environment migration", "tar.gz export/import",
  ];

  return (
    <div className="relative overflow-hidden border-y border-zinc-800 bg-zinc-900/30 py-4">
      <div className="marquee-track flex gap-12 whitespace-nowrap">
        {[...items, ...items].map((item, i) => (
          <span key={i} className="font-mono text-sm text-zinc-600">
            [{item}]
          </span>
        ))}
      </div>
    </div>
  );
}
