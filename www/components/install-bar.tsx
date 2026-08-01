"use client";import {useState} from "react";const T=[{id:"shell",l:"Shell",c:"curl -fsSL https://raw.githubusercontent.com/oneinitAI/oneinit/main/install.sh | sh",n:"Zero prerequisites. Auto-detects OS and architecture."},{id:"npm",l:"npm",c:"npm install -g oneinit",n:"Node.js 14+. PATH handled automatically."},{id:"source",l:"Source",c:"git clone https://github.com/oneinitAI/oneinit.git\ncd oneinit && cargo build --release",n:"Rust 1.94+. Binary at target/release/oneinit."}];export function InstallBar(){const[a,setA]=useState(0);const[c,setC]=useState(false);const copy=()=>{navigator.clipboard.writeText(T[a].c);setC(true);setTimeout(()=>setC(false),2000)};return(
<section id="install" className="border-t border-white/[0.04] py-24 md:py-32" data-aos="fade-up"><div className="mx-auto max-w-[750px] px-6 text-center">
<span className="font-mono text-xs uppercase tracking-[0.3em] text-emerald-500">Install</span>
<h2 className="mt-3 mb-2 text-3xl font-bold text-white md:text-5xl">One line. <span className="text-zinc-600">Done.</span></h2>
<p className="mb-10 text-zinc-500">Pick your method. All install the same binary.</p>
<div className="glass overflow-hidden rounded-2xl">
<div className="flex border-b border-white/[0.04]">{T.map((t,i)=>(
<button key={t.id} onClick={()=>setA(i)} className={`flex-1 py-3 font-mono text-sm transition-all ${a===i?"text-emerald-600 border-b-2 border-emerald-600 bg-emerald-600/[0.02]":"text-zinc-600 hover:text-zinc-300"}`}>{t.l}</button>))}</div>
<div className="relative p-6 text-left">
<button onClick={copy} className="absolute right-4 top-4 glass-hover rounded-lg px-3 py-1.5 font-mono text-xs text-zinc-400 transition-all">{c?"copied!":"copy"}</button>
<pre className="font-mono text-sm leading-relaxed text-zinc-200"><code>{T[a].c.split("\n").map((l,i)=><div key={i}><span className="select-none text-emerald-600">$ </span>{l}</div>)}</code></pre>
<p className="mt-3 text-xs text-zinc-600">{T[a].n}</p></div></div></div></section>)}