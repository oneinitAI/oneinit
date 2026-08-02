"use client";
import { useLang } from "./lang-provider";

const CODE = `# 团队环境定义 team.yaml（fork 模板后修改）
team:
  name: "WebTeam"
  signing_key: "d8ee1d1c..."   # 可选：Ed25519 公钥

envs:
  node: "20"                    # → node20
  python: "3.11"               # → python3.11

mirrors:
  pip: "tsinghua"
  npm: "npmmirror"

env_vars:
  NODE_ENV: "development"

config_files:
  - path: "{{user_home}}/.npmrc"
    template: "registry={{mirror_npm}}"`;

export function TeamEnv() {
  const { t } = useLang();
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <div className="grid grid-cols-1 gap-12 md:grid-cols-2 items-center">
          <div data-aos="fade-right">
            <span className="font-mono text-xs uppercase tracking-[0.3em] text-violet-500">{t("team.badge")}</span>
            <h2 className="mt-3 mb-4 text-3xl font-bold text-white md:text-5xl">{t("team.title1")}<br /><span className="text-zinc-600">{t("team.title2")}</span></h2>
            <p className="mb-6 leading-relaxed text-zinc-400 max-w-[440px]">
              {t("team.desc")}
              <br /><br />
              {t("team.security")}
            </p>
            <div className="flex flex-wrap gap-2 text-sm text-zinc-600 font-mono">
              <span>oneinit team add &lt;url&gt;</span><span>·</span><span>team sync --force</span><span>·</span><span>team status</span>
            </div>
          </div>
          <div data-aos="fade-left" className="overflow-hidden rounded-xl border border-white/[0.06] bg-zinc-900/80">
            <div className="flex items-center gap-1.5 border-b border-white/[0.04] px-4 py-2.5">
              <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" /><span className="h-2.5 w-2.5 rounded-full bg-zinc-700" /><span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
              <span className="ml-2 font-mono text-[11px] text-zinc-600">team.yaml</span>
            </div>
            <pre className="terminal-scroll overflow-x-auto p-5 font-mono text-[13px] leading-relaxed text-zinc-300"><code>{CODE}</code></pre>
          </div>
        </div>
      </div>
    </section>
  );
}
