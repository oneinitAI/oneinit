"use client";

import { SceneController } from "@/components/promo/SceneController";

export default function PromoPage() {
  return (
    <div className="h-screen w-screen bg-[#0d0d0d] text-[#e4e4e4] flex flex-col overflow-hidden font-mono select-none">
      {/* Top Navbar */}
      <header className="h-[44px] bg-[#141414] border-b border-[#2a2a2a] flex items-center px-[18px] gap-4 shrink-0">
        <div className="flex items-center gap-2 font-semibold text-[15px] text-[#f0f0f0]">
          <svg className="w-[18px] h-[18px] text-emerald-500" fill="currentColor" viewBox="0 0 24 24">
            <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/>
          </svg>
          <span className="bg-gradient-to-r from-emerald-500 to-teal-400 bg-clip-text text-transparent">Vibe</span>
          <span className="text-[#eee]">Code</span>
        </div>
        <div className="flex items-center gap-1 ml-2 text-[13px] text-[#555] border-l border-[#2a2a2a] pl-3">
          <span className="text-emerald-500 text-xs">[project]</span>
          <span className="text-[#999]">~/projects/data-analysis</span>
        </div>
        <div className="ml-auto flex items-center gap-1.5">
          <button className="bg-transparent border-none text-[#9e9e9e] text-[13px] px-3 py-1.5 rounded-md cursor-pointer hover:bg-[#2a2a2a] hover:text-[#eee] transition-all flex items-center gap-1.5">
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
            Search
          </button>
          <button className="bg-transparent border-none text-[#9e9e9e] text-[13px] px-3 py-1.5 rounded-md cursor-pointer hover:bg-[#2a2a2a] hover:text-[#eee] transition-all flex items-center gap-1.5">
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>
            Terminal
          </button>
          <button className="bg-transparent border-none text-[#9e9e9e] text-[13px] px-3 py-1.5 rounded-md cursor-pointer hover:bg-[#2a2a2a] hover:text-[#eee] transition-all flex items-center gap-1.5">
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
            main
          </button>
          <button className="bg-emerald-600 border-none text-white text-[13px] px-4 py-1.5 rounded-md cursor-pointer hover:bg-emerald-500 transition-all font-medium flex items-center gap-1.5">
            <svg className="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
            Run
          </button>
        </div>
      </header>

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden">
        <section className="flex-1 bg-[#0f0f0f] flex flex-col min-w-0">
          {/* Chat header */}
          <div className="px-5 py-3 border-b border-[#222] flex items-center gap-3 shrink-0">
            <div className="text-sm font-medium text-[#f0f0f0]">
              <span className="text-emerald-500 mr-2">#</span>
              Data Analysis Script
            </div>
            <div className="text-xs text-emerald-500 flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 bg-emerald-500 rounded-full inline-block animate-pulse" />
              Online / Context 2.1k tokens
            </div>
            <div className="ml-auto flex gap-1">
              <button className="bg-transparent border-none text-[#666] px-2 py-1 rounded-md cursor-pointer hover:bg-[#2a2a2a] hover:text-[#eee] text-sm transition-all">...</button>
            </div>
          </div>

          {/* Messages area */}
          <div className="flex-1 overflow-y-auto px-6 py-4 terminal-scroll">
            <SceneController />
          </div>

          {/* Input area */}
          <div className="px-5 pb-4 pt-3 border-t border-[#222] shrink-0 bg-[#0f0f0f]">
            <div className="flex items-end gap-2 bg-[#181818] border border-[#2e2e2e] rounded-xl px-3.5 py-2 transition-all focus-within:border-emerald-600">
              <button className="bg-transparent border-none text-[#666] p-1 cursor-pointer hover:text-[#ccc] hover:bg-[#2a2a2a] rounded-md text-base transition-all">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"/></svg>
              </button>
              <textarea
                className="flex-1 bg-transparent border-none outline-none text-[#e0e0e0] text-[13px] font-mono resize-none py-1.5 min-h-[24px] max-h-[100px] leading-relaxed"
                placeholder="Ask Codex..."
                rows={1}
                readOnly
              />
              <button className="bg-emerald-600 border-none text-white w-[34px] h-[34px] rounded-full cursor-pointer flex items-center justify-center text-[15px] transition-all hover:bg-emerald-500 hover:scale-[1.04] active:scale-95 shrink-0">
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
              </button>
            </div>
            <div className="flex justify-between px-1 pt-1.5 text-[11px] text-[#444]">
              <span><span className="text-emerald-500">Ctrl+Enter</span> to send</span>
              <span>Context 2.1k / 8k</span>
            </div>
          </div>
        </section>
      </div>

      {/* Bottom status bar */}
      <footer className="h-[28px] bg-[#0e0e0e] border-t border-[#222] flex items-center px-[18px] text-xs text-[#6a6a6a] gap-5 shrink-0">
        <span className="flex items-center gap-1.5">
          <span className="w-2 h-2 bg-emerald-500 rounded-full inline-block" />
          Ready
        </span>
        <span className="text-[#2a2a2a]">|</span>
        <span className="flex items-center gap-1.5">
          <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
          main
        </span>
        <span className="text-[#2a2a2a]">|</span>
        <span className="flex items-center gap-1.5">
          <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>
          UTF-8
        </span>
        <span className="ml-auto flex items-center gap-4">
          <span>Ln 12, Col 34</span>
          <span className="flex items-center gap-1.5">
            <svg className="w-3 h-3 text-emerald-500" fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>
            Codex v2.1
          </span>
        </span>
      </footer>
    </div>
  );
}
