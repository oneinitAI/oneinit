"use client";
const RECIPE = `name: node20
version: "20.18.1"
description: "Node.js 20 LTS"

platforms:
  windows:
    url: "https://nodejs.org/..."
    sha256: "56e5aacd..."
    install_type: "zip_extract"

post_install:
  config_files:
    - path: ".npmrc"
      template: "registry={{mirror_npm}}"

maintainer:
  github: "BG4JTS"`;

export function CommunityRecipe() {
  return (
    <section className="border-t border-white/[0.04] py-24 md:py-32">
      <div className="mx-auto max-w-[1100px] px-6">
        <div className="grid grid-cols-1 gap-12 md:grid-cols-2 items-center">
          <div data-aos="fade-right">
            <span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">Community Registry</span>
            <h2 className="mt-3 mb-4 text-3xl font-bold text-white md:text-5xl">Publish YAML.<br /><span className="text-zinc-600">Anyone installs.</span></h2>
            <p className="mb-6 leading-relaxed text-zinc-400 max-w-[440px]">
              Like npm, but for dev tools. Write a recipe. Push to GitHub. One command installs it anywhere.
              <br /><br />
              Built-in security: every recipe is SHA256-verified before installation. Users confirm before anything runs.
            </p>
            <div className="flex gap-4 text-sm text-zinc-600 font-mono">
              <span>zip_extract</span><span>·</span><span>tar_extract</span><span>·</span><span>exe_silent</span><span>·</span><span>msi_install</span><span>·</span><span>pkg_install</span>
            </div>
          </div>
          <div data-aos="fade-left" className="overflow-hidden rounded-xl border border-white/[0.06] bg-zinc-900/80">
            <div className="flex items-center gap-1.5 border-b border-white/[0.04] px-4 py-2.5">
              <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" /><span className="h-2.5 w-2.5 rounded-full bg-zinc-700" /><span className="h-2.5 w-2.5 rounded-full bg-zinc-700" />
              <span className="ml-2 font-mono text-[11px] text-zinc-600">recipe.yaml</span>
            </div>
            <pre className="terminal-scroll overflow-x-auto p-5 font-mono text-[13px] leading-relaxed text-zinc-300"><code>{RECIPE}</code></pre>
          </div>
        </div>
      </div>
    </section>
  );
}
