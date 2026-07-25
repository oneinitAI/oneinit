"use client"; import { useState,useEffect,useRef } from "react";
const L = [{t:"$ oneinit install python3.11",c:""},{t:"[OK] Download complete (10.7 MB)",c:"text-zinc-500"},{t:"[OK] SHA256 verified",c:"text-zinc-500"},{t:"[OK] Mirror: Tsinghua configured",c:"text-emerald-500"},{t:"[OK] PATH updated",c:"text-zinc-500"},{t:"",c:""},{t:"$ python --version",c:""},{t:"Python 3.11.9",c:"text-white font-bold"}];
export function Terminal(){const[vis,setVis]=useState(0);const sr=useRef<HTMLDivElement>(null);
useEffect(()=>{let i=0;const t=setInterval(()=>{setVis(++i);if(i>=L.length)clearInterval(t)},350);return()=>clearInterval(t)},[]);
useEffect(()=>{if(sr.current)sr.current.scrollTop=sr.current.scrollHeight},[vis]);
return(<div className="glass overflow-hidden rounded-2xl shadow-2xl w-full max-w-[460px]">
  <div className="flex items-center gap-2 border-b border-white/[0.04] px-4 py-2.5">
    <span className="h-3 w-3 rounded-full bg-zinc-700"/><span className="h-3 w-3 rounded-full bg-zinc-700"/><span className="h-3 w-3 rounded-full bg-zinc-700"/>
    <span className="ml-2 font-mono text-[11px] text-zinc-600">Terminal · oneinit</span>
  </div>
  <div ref={sr} className="terminal-scroll h-[340px] overflow-y-auto p-4 font-mono text-[13px] leading-relaxed">
    {L.slice(0,vis).map((l,i)=>(<div key={i} className={l.c||"text-zinc-300"}>
      {!l.c&&l.t.startsWith("$")&&<span className="text-emerald-500 select-none">$ </span>}{l.t}
      {i===vis-1&&vis<L.length&&<span className="inline-block h-4 w-2 bg-emerald-500 animate-pulse align-middle ml-0.5"/>}
    </div>))}
  </div>
</div>)}