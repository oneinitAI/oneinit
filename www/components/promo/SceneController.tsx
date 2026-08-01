"use client";

import { useState, useEffect, useCallback } from "react";
import { AnimatePresence } from "motion/react";
import { ChatBubble } from "./TypewriterLine";
import { TransitionOverlay } from "./TransitionOverlay";
import { FinalCTAScreen } from "./FinalCTAScreen";
import { scenes, type ScriptLine } from "./data/script";

type Phase = "playing" | "transitioning" | "final";

export function SceneController() {
  const [phase, setPhase] = useState<Phase>("playing");
  const [sceneIndex, setSceneIndex] = useState(0);
  const [lineIndex, setLineIndex] = useState(0);
  const [visibleLines, setVisibleLines] = useState<{ scene: number; line: number }[]>([
    { scene: 0, line: 0 },
  ]);
  const [completedLines, setCompletedLines] = useState<Set<string>>(new Set());

  const scene = scenes[sceneIndex];
  const lineKey = (s: number, l: number) => `${s}-${l}`;

  const handleLineComplete = useCallback(
    (sceneNum: number, lineNum: number) => {
      const key = lineKey(sceneNum, lineNum);
      setCompletedLines((prev) => new Set(prev).add(key));

      const currentScene = scenes[sceneNum];
      if (!currentScene) return;

      if (lineNum >= currentScene.lines.length - 1) {
        if (sceneNum === 2) {
          setTimeout(() => setPhase("transitioning"), 1000);
        } else if (sceneNum === scenes.length - 1) {
          setTimeout(() => setPhase("final"), 1200);
        } else {
          setTimeout(() => {
            setSceneIndex((s) => s + 1);
            setLineIndex(0);
            setVisibleLines([]);
            setCompletedLines(new Set());
          }, 800);
        }
      } else {
        const nextLine = lineNum + 1;
        setTimeout(() => {
          setLineIndex(nextLine);
          setVisibleLines((prev) => [...prev, { scene: sceneNum, line: nextLine }]);
        }, 300);
      }
    },
    []
  );

  useEffect(() => {
    if (phase !== "playing") return;
    setVisibleLines([{ scene: sceneIndex, line: 0 }]);
    setLineIndex(0);
    setCompletedLines(new Set());
  }, [sceneIndex, phase]);

  const handleTransitionComplete = useCallback(() => {
    setPhase("playing");
    setSceneIndex(3);
    setLineIndex(0);
    setVisibleLines([]);
    setCompletedLines(new Set());
  }, []);

  const handleReplay = useCallback(() => {
    setPhase("playing");
    setSceneIndex(0);
    setLineIndex(0);
    setVisibleLines([]);
    setCompletedLines(new Set());
  }, []);

  return (
    <div className="relative">
      <AnimatePresence mode="wait">
        {phase === "transitioning" && (
          <TransitionOverlay key="transition" onComplete={handleTransitionComplete} />
        )}

        {phase === "final" && <FinalCTAScreen key="final" onReplay={handleReplay} />}

        {phase === "playing" && (
          <div key={`scene-${sceneIndex}`} className="flex flex-col gap-3.5">
            {visibleLines.map(({ scene: s, line: l }) => {
              const key = lineKey(s, l);
              const isComplete = completedLines.has(key);
              const line = scenes[s]?.lines[l];
              if (!line) return null;

              return (
                <div key={key}>
                  {isComplete ? (
                    <StaticBubble line={line} />
                  ) : (
                    <ChatBubble
                      line={line}
                      onComplete={() => handleLineComplete(s, l)}
                    />
                  )}
                </div>
              );
            })}
          </div>
        )}
      </AnimatePresence>
    </div>
  );
}

/** Renders a completed line as a static bubble */
function StaticBubble({ line }: { line: ScriptLine }) {
  const isUser = line.type === "user";
  const isAI = line.type === "ai";
  const hasAvatar = isUser || isAI;

  const bubbleBg: Record<string, string> = {
    user: "ml-auto bg-[#0a1a14] border border-[#1a3028]",
    ai: "mr-auto bg-[#1a1a1a] border border-[#282828]",
    terminal: "mr-auto bg-[#1a1a1a]/60 border border-[#282828]",
    error: "mr-auto bg-[#2a1015] border border-[#4a2020]",
    success: "mr-auto bg-[#0a1a10] border border-[#1a4020]",
    system: "mx-auto bg-transparent border-0 text-center",
  };

  const textColor: Record<string, string> = {
    user: "text-[#d4d4d4]",
    ai: "text-[#d4d4d4]",
    terminal: "text-[#e0af68]",
    error: "text-[#f7768e]",
    success: "text-[#4ade80]",
    system: "text-[#546e7a] italic text-xs",
  };

  if (line.type === "system") {
    return (
      <div className="flex justify-center py-1.5">
        <div className="text-xs leading-relaxed whitespace-pre-wrap text-[#8899aa] max-w-[90%] border border-dashed border-[#2a2a2a] rounded-lg px-4 py-2 bg-[#141414]/50">
          {line.text}
        </div>
      </div>
    );
  }

  return (
    <div className={`flex items-start gap-3 ${isUser ? "flex-row-reverse" : "flex-row"}`}>
      {hasAvatar ? (
        isUser ? (
          <div className="w-[30px] h-[30px] rounded-full bg-[#3d3d3d] text-[#ccc] flex items-center justify-center text-[13px] font-semibold shrink-0 select-none">
            U
          </div>
        ) : (
          <div className="w-[30px] h-[30px] rounded-full bg-gradient-to-br from-emerald-600 to-teal-500 text-white flex items-center justify-center shrink-0 select-none">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
          </div>
        )
      ) : (
        <div className="w-[30px] shrink-0" />
      )}

      <div className={`rounded-xl px-4 py-2.5 max-w-[85%] text-[13.5px] leading-relaxed ${bubbleBg[line.type] || ""}`}>
        {line.prefix && (
          <span className={`${line.type === "error" ? "text-[#f7768e]" : line.type === "success" ? "text-[#4ade80]" : "text-[#565f89]"} mr-1.5 select-none`}>
            {line.prefix}
          </span>
        )}
        <span className={`whitespace-pre-wrap ${textColor[line.type] || "text-[#d4d4d4]"}`}>
          {line.text}
        </span>
      </div>
    </div>
  );
}
