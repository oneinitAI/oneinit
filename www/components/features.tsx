"use client";const I=[{t:"Auto Mirror Config",d:"pip uses Tsinghua. npm uses npmmirror. No manual setup. No config files. Works on every install.",n:"01"},{t:"7 Language Detectors",d:"Scan any machine: Python, Node, Git, Rust, Go, Java, Docker. Plus custom detectors via scan_config.yaml.",n:"02"},{t:"Environment Migration",d:"Export your entire setup as tar.gz. Import on a new machine. Tools, configs, package lists — everything restored.",n:"03"},{t:"Clean Uninstall",d:"SQLite manifest tracks every PATH entry and config file. Uninstall removes everything. No leftovers. No guessing.",n:"04"}];export function Features(){return(
<section className="border-t border-white/[0.04] py-24 md:py-32"><div className="mx-auto max-w-[1100px] px-6">
<div className="grid grid-cols-1 gap-6 md:grid-cols-2">{I.map((item,i)=>(
<div key={i} data-aos="fade-up" data-aos-delay={i*150} className="glass rounded-2xl p-8 transition-all glass-hover hover:-translate-y-1 group">
<div className="mb-4 font-mono text-sm text-emerald-600">{item.n}</div>
<h3 className="mb-2 text-xl font-bold text-white">{item.t}</h3>
<p className="leading-relaxed text-zinc-400">{item.d}</p></div>
))}</div></div></section>)}