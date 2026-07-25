"use client";

import { useEffect, useRef, useState } from "react";

const CHARS = "!<>-_\\/[]{}—=+*^?#________";

interface Props {
  text: string;
  className?: string;
  trigger?: boolean;
}

export function TextScramble({ text, className = "", trigger = true }: Props) {
  const elRef = useRef<HTMLSpanElement>(null);
  const [frame, setFrame] = useState(0);
  const queue = useRef<{ from: string; to: string; start: number; end: number }[]>([]);
  const frameRef = useRef(0);

  useEffect(() => {
    if (!trigger || !elRef.current) return;

    const resolveAfter = (ms: number) => new Promise(r => setTimeout(r, ms));

    const setText = (newText: string) => {
      const oldText = elRef.current?.textContent ?? "";
      const length = Math.max(oldText.length, newText.length);
      const from = oldText.padEnd(length, " ");
      queue.current.push({ from, to: newText.padEnd(length, " "), start: frameRef.current, end: frameRef.current + 15 });
    };

    const update = () => {
      let output = "";
      let complete = 0;
      const f = frameRef.current;
      for (const q of queue.current) {
        const { from, to, start, end } = q;
        if (f >= end) {
          complete++;
          output += to;
        } else if (f < start) {
          output += from;
        } else {
          const progress = (f - start) / (end - start);
          let str = "";
          for (let i = 0; i < to.length; i++) {
            if (i < progress * to.length) str += to[i];
            else str += CHARS[Math.floor(Math.random() * CHARS.length)];
          }
          output += str;
        }
      }
      elRef.current!.textContent = output;
      if (complete === queue.current.length) queue.current = [];
    };

    const loop = () => {
      update();
      frameRef.current++;
      requestAnimationFrame(loop);
    };
    const anim = requestAnimationFrame(loop);

    (async () => {
      await resolveAfter(200);
      setText(text);
    })();

    return () => cancelAnimationFrame(anim);
  }, [text, trigger]);

  return <span ref={elRef} className={className}>{text}</span>;
}
