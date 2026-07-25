"use client";

import { useRef, useState, useCallback, type ReactNode } from "react";
import { motion, useReducedMotion } from "motion/react";

interface Props {
  children: ReactNode;
  className?: string;
  href?: string;
  onClick?: () => void;
  variant?: "primary" | "secondary" | "ghost";
}

export function MagneticButton({
  children,
  className = "",
  href,
  onClick,
  variant = "primary",
}: Props) {
  const btnRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ x: 0, y: 0 });
  const [hovering, setHovering] = useState(false);
  const reduce = useReducedMotion();

  const handleMove = useCallback(
    (e: React.MouseEvent) => {
      if (reduce || !btnRef.current) return;
      const rect = btnRef.current.getBoundingClientRect();
      const x = (e.clientX - rect.left - rect.width / 2) * 0.35;
      const y = (e.clientY - rect.top - rect.height / 2) * 0.35;
      setPos({ x, y });
    },
    [reduce]
  );

  const handleLeave = useCallback(() => {
    setPos({ x: 0, y: 0 });
    setHovering(false);
  }, []);

  const variants: Record<string, string> = {
    primary:
      "bg-emerald-500 text-zinc-950 shadow-lg shadow-emerald-500/20 hover:shadow-emerald-500/40",
    secondary:
      "border border-zinc-700 text-zinc-200 bg-zinc-900/50 hover:border-emerald-500/50",
    ghost:
      "text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900/50",
  };

  const content = (
    <motion.div
      ref={btnRef}
      onMouseMove={handleMove}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={handleLeave}
      animate={reduce ? undefined : { x: pos.x, y: pos.y }}
      transition={{ type: "spring", stiffness: 150, damping: 15, mass: 0.1 }}
      className={`inline-flex cursor-pointer items-center gap-2 rounded-xl px-6 py-3 text-sm font-medium transition-all active:scale-[0.97] ${variants[variant]} ${className}`}
      onClick={onClick}
    >
      {children}
      {!reduce && hovering && variant === "primary" && (
        <motion.span
          initial={{ scale: 0, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          className="absolute inset-0 rounded-xl bg-gradient-to-r from-emerald-500/0 via-white/10 to-emerald-500/0"
          transition={{ duration: 0.4 }}
        />
      )}
    </motion.div>
  );

  if (href) {
    return (
      <a href={href} target={href.startsWith("http") ? "_blank" : undefined} rel="noopener noreferrer">
        {content}
      </a>
    );
  }

  return content;
}
