"use client";

import { useState, useEffect, useRef } from "react";
import { motion } from "motion/react";
import type { ScriptLine } from "./data/script";
import { MarkdownContent } from "./MarkdownContent";

interface ChatBubbleProps {
  line: ScriptLine;
  onComplete?: () => void;
}

/* Style maps */

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
  system: "text-[#8899aa] text-xs",
};

/* Avatar */
function Avatar({ type }: { type: string }) {
  if (type === "user") {
    return (
      <div className="w-[30px] h-[30px] rounded-full bg-[#3d3d3d] text-[#ccc] flex items-center justify-center text-[13px] font-semibold shrink-0 select-none">
        U
      </div>
    );
  }
  if (type === "ai") {
    return (
      <div className="w-[30px] h-[30px] rounded-full bg-gradient-to-br from-emerald-600 to-teal-500 text-white flex items-center justify-center shrink-0 select-none">
        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
      </div>
    );
  }
  return <div className="w-[30px] shrink-0" />;
}

/* Main component */
export function ChatBubble({ line, onComplete }: ChatBubbleProps) {
  const [displayText, setDisplayText] = useState("");
  const [isComplete, setIsComplete] = useState(false);
  const [started, setStarted] = useState(false);
  const delayTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const typeTimer = useRef<ReturnType<typeof setInterval> | null>(null);
  const onCompleteRef = useRef(onComplete);
  onCompleteRef.current = onComplete;

  useEffect(() => {
    setDisplayText("");
    setIsComplete(false);
    setStarted(false);

    const delay = line.delay ?? 0;
    const speed = line.speed ?? 25;
    let charIndex = 0;

    delayTimer.current = setTimeout(() => {
      setStarted(true);
      typeTimer.current = setInterval(() => {
        charIndex++;
        if (charIndex <= line.text.length) {
          setDisplayText(line.text.slice(0, charIndex));
        } else {
          if (typeTimer.current) clearInterval(typeTimer.current);
          setIsComplete(true);
          onCompleteRef.current?.();
        }
      }, speed);
    }, delay);

    return () => {
      if (delayTimer.current) clearTimeout(delayTimer.current);
      if (typeTimer.current) clearInterval(typeTimer.current);
    };
  }, [line.text, line.delay, line.speed]);

  const isUser = line.type === "user";
  const isAI = line.type === "ai";
  const hasAvatar = isUser || isAI;
  const hasBubble = line.type !== "system";
  const showRenderedMarkdown = isComplete && isAI;

  if (line.type === "system") {
    return (
      <motion.div
        initial={{ opacity: 0, y: 4 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35 }}
        className="flex justify-center py-1.5"
      >
        <div className="text-xs leading-relaxed whitespace-pre-wrap text-[#8899aa] max-w-[90%] border border-dashed border-[#2a2a2a] rounded-lg px-4 py-2 bg-[#141414]/50">
          {isComplete ? line.text : displayText}
          {!isComplete && (
            <span className="inline-block w-1.5 h-3.5 ml-0.5 align-middle animate-pulse rounded-sm bg-[#8899aa]" />
          )}
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: "easeOut" }}
      className={`flex items-start gap-3 ${isUser ? "flex-row-reverse" : "flex-row"}`}
    >
      {hasAvatar && <Avatar type={line.type} />}
      {!hasAvatar && <div className="w-[30px] shrink-0" />}

      <div className={`rounded-xl px-4 py-2.5 max-w-[85%] text-[13.5px] leading-relaxed ${hasBubble ? bubbleBg[line.type] || "" : ""}`}>
        {line.prefix && !showRenderedMarkdown && (
          <span className={`${line.type === "error" ? "text-[#f7768e]" : line.type === "success" ? "text-[#4ade80]" : "text-[#565f89]"} mr-1.5 select-none`}>
            {line.prefix}
          </span>
        )}

        <span className={`whitespace-pre-wrap ${textColor[line.type] || "text-[#d4d4d4]"}`}>
          {showRenderedMarkdown ? (
            <MarkdownContent text={line.text} />
          ) : (
            displayText
          )}
        </span>

        {!isComplete && (
          <span className={`inline-block w-1.5 h-3.5 ml-0.5 align-middle animate-pulse rounded-sm ${
            isUser ? "bg-emerald-500" : "bg-emerald-500"
          }`} />
        )}
      </div>
    </motion.div>
  );
}
