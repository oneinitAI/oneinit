"use client";

import { useEffect, useRef, useCallback } from "react";
import { useReducedMotion } from "motion/react";

interface Particle {
  x: number; y: number;
  ox: number; oy: number;
  vx: number; vy: number;
  r: number;
  hue: number;
}

export function InteractiveBg() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const mouse = useRef({ x: -200, y: -200 });
  const particles = useRef<Particle[]>([]);
  const reduce = useReducedMotion();

  const init = useCallback(() => {
    const canvas = canvasRef.current; if (!canvas) return;
    const w = canvas.width = window.innerWidth;
    const h = canvas.height = window.innerHeight;
    const count = Math.floor((w * h) / 12000);

    particles.current = Array.from({ length: count }, () => ({
      x: Math.random() * w, y: Math.random() * h,
      ox: Math.random() * w, oy: Math.random() * h,
      vx: (Math.random() - 0.5) * 0.3, vy: (Math.random() - 0.5) * 0.3,
      r: Math.random() * 1.5 + 0.5,
      hue: 160 + Math.random() * 20,
    }));
  }, []);

  useEffect(() => {
    if (reduce) return;
    init();

    const canvas = canvasRef.current; if (!canvas) return;
    const ctx = canvas.getContext("2d"); if (!ctx) return;

    const handleMove = (e: MouseEvent) => {
      mouse.current = { x: e.clientX, y: e.clientY };
    };
    const handleResize = () => init();

    window.addEventListener("mousemove", handleMove, { passive: true });
    window.addEventListener("resize", handleResize);

    let frame: number;

    function draw() {
      const w = canvas!.width, h = canvas!.height;
      const mx = mouse.current.x, my = mouse.current.y;

      ctx!.clearRect(0, 0, w, h);

      // Draw connections first (behind particles)
      for (let i = 0; i < particles.current.length; i++) {
        for (let j = i + 1; j < particles.current.length; j++) {
          const a = particles.current[i], b = particles.current[j];
          const dx = a.x - b.x, dy = a.y - b.y;
          const dist = Math.sqrt(dx * dx + dy * dy);
          if (dist < 100) {
            const alpha = (1 - dist / 100) * 0.08;
            ctx!.beginPath();
            ctx!.moveTo(a.x, a.y);
            ctx!.lineTo(b.x, b.y);
            ctx!.strokeStyle = `rgba(5,150,105,${alpha})`;
            ctx!.lineWidth = 0.5;
            ctx!.stroke();
          }
        }
      }

      // Update and draw particles
      for (const p of particles.current) {
        // Mouse attraction
        const ddx = mx - p.x, ddy = my - p.y;
        const md = Math.sqrt(ddx * ddx + ddy * ddy);
        if (md < 200 && md > 0) {
          const force = (200 - md) / 200 * 0.02;
          p.vx += ddx / md * force;
          p.vy += ddy / md * force;
        }

        // Return to origin gently
        p.vx += (p.ox - p.x) * 0.0005;
        p.vy += (p.oy - p.y) * 0.0005;

        // Damping
        p.vx *= 0.98; p.vy *= 0.98;

        p.x += p.vx; p.y += p.vy;

        // Draw
        ctx!.beginPath();
        ctx!.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx!.fillStyle = `hsla(${p.hue},60%,50%,0.3)`;
        ctx!.fill();
      }

      frame = requestAnimationFrame(draw);
    }

    frame = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("resize", handleResize);
    };
  }, [reduce, init]);

  return (
    <>
      {/* Animated gradient blobs behind canvas */}
      <div className="fixed inset-0 pointer-events-none z-0" aria-hidden="true">
        <div className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full bg-emerald-600/[0.03] blur-[120px]"
          style={{ animation: "blob1 20s ease-in-out infinite" }} />
        <div className="absolute top-2/3 right-1/4 w-[500px] h-[500px] rounded-full bg-emerald-600/[0.02] blur-[100px]"
          style={{ animation: "blob2 25s ease-in-out infinite" }} />
        <div className="absolute bottom-1/3 left-1/2 w-[400px] h-[400px] rounded-full bg-white/[0.01] blur-[80px]"
          style={{ animation: "blob3 18s ease-in-out infinite" }} />
      </div>

      {/* Particle canvas */}
      <canvas ref={canvasRef} className="fixed inset-0 pointer-events-none z-[1]" aria-hidden="true" />

      <style jsx>{`
        @keyframes blob1 {
          0%,100% { transform: translate(0,0) scale(1); }
          33% { transform: translate(100px,-50px) scale(1.2); }
          66% { transform: translate(-50px,80px) scale(0.8); }
        }
        @keyframes blob2 {
          0%,100% { transform: translate(0,0) scale(1); }
          50% { transform: translate(-80px,-30px) scale(1.3); }
        }
        @keyframes blob3 {
          0%,100% { transform: translate(0,0) scale(1); }
          25% { transform: translate(60px,40px) scale(0.7); }
          75% { transform: translate(-40px,-60px) scale(1.4); }
        }
      `}</style>
    </>
  );
}
