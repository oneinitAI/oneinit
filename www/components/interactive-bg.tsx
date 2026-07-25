"use client";

import { useEffect, useRef, useState } from "react";
import { useReducedMotion } from "motion/react";

export function InteractiveBg() {
  const vantaRef = useRef<HTMLDivElement>(null);
  const [effect, setEffect] = useState<any>(null);
  const reduce = useReducedMotion();

  useEffect(() => {
    if (reduce || !vantaRef.current) return;
    let instance: any = null;
    import("vanta/dist/vanta.net.min").then((NET) => {
      instance = NET.default?.({
        el: vantaRef.current!,
        mouseControls: true,
        touchControls: true,
        gyroControls: false,
        minHeight: 200.00,
        minWidth: 200.00,
        scale: 1.0,
        scaleMobile: 1.0,
        color: 0x059669,
        backgroundColor: 0x0a0a0f,
        points: 12.0,
        maxDistance: 20.0,
        spacing: 18.0,
        showDots: false,
      });
      setEffect(instance);
    });

    return () => { if (instance) instance.destroy(); };
  }, [reduce]);

  return (
    <div
      ref={vantaRef}
      className="fixed inset-0 z-0 pointer-events-none"
      style={{ width: "100vw", height: "100vh" }}
      aria-hidden="true"
    />
  );
}
