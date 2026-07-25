"use client";

import { useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";

export function CustomCursor() {
  const ref = useRef<HTMLDivElement>(null);
  const reduce = useReducedMotion();

  useEffect(() => {
    if (reduce) return;
    const cursor = ref.current;
    if (!cursor) return;

    const update = (e: MouseEvent) => {
      cursor.style.left = `${e.clientX - 10}px`;
      cursor.style.top = `${e.clientY - 10}px`;
    };
    const enter = (e: MouseEvent) => {
      const el = e.target as HTMLElement;
      if (el.closest("a, button, [data-clickable], input, textarea")) {
        cursor.classList.add("hover");
      }
    };
    const leave = () => cursor.classList.remove("hover");

    document.addEventListener("mousemove", update);
    document.addEventListener("mouseover", enter);
    document.addEventListener("mouseout", leave);

    return () => {
      document.removeEventListener("mousemove", update);
      document.removeEventListener("mouseover", enter);
      document.removeEventListener("mouseout", leave);
    };
  }, [reduce]);

  if (reduce) return null;

  return <div ref={ref} className="custom-cursor" />;
}
