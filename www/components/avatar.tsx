"use client";
import { useState } from "react";

const PALETTE = [
  "#10b981",
  "#8b5cf6",
  "#f59e0b",
  "#ef4444",
  "#06b6d4",
  "#ec4899",
  "#84cc16",
  "#6366f1",
];

/**
 * 贡献者头像：加载失败（账号被禁用 / 网络问题）时回退为「首字母圆形」，
 * 避免破图。
 */
export function Avatar({
  src,
  alt,
  size = 32,
  className = "",
}: {
  src: string;
  alt: string;
  size?: number;
  className?: string;
}) {
  const [failed, setFailed] = useState(false);
  const initials = (alt || "?").slice(0, 2).toUpperCase();
  const color =
    PALETTE[
      (alt || "").split("").reduce((s, c) => s + c.charCodeAt(0), 0) % PALETTE.length
    ];

  if (failed || !src) {
    return (
      <div
        className={`flex shrink-0 items-center justify-center rounded-full font-bold text-white ${className}`}
        style={{
          width: size,
          height: size,
          background: `linear-gradient(135deg, ${color}, ${color}99)`,
          fontSize: Math.max(10, size * 0.38),
        }}
      >
        {initials}
      </div>
    );
  }

  return (
    <img
      src={src}
      alt={alt}
      width={size}
      height={size}
      loading="lazy"
      onError={() => setFailed(true)}
      className={`shrink-0 rounded-full border border-white/10 ${className}`}
      style={{ width: size, height: size }}
    />
  );
}
