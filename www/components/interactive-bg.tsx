"use client";

import { useCallback } from "react";
import Particles from "@tsparticles/react";
import { loadSlim } from "@tsparticles/slim";
import type { Engine } from "@tsparticles/engine";
import { useReducedMotion } from "motion/react";

export function InteractiveBg() {
  const reduce = useReducedMotion();
  const init = useCallback(async (engine: Engine) => { await loadSlim(engine); }, []);

  if (reduce) return null;

  return (
    <Particles
      id="tsparticles"
      init={init}
      className="fixed inset-0 z-0 pointer-events-none"
      options={{
        fullScreen: false,
        fpsLimit: 60,
        particles: {
          number: { value: 80, density: { enable: true } },
          color: { value: ["#059669", "#10b981", "#34d399", "#6ee7b7"] },
          links: {
            enable: true,
            distance: 150,
            color: "#059669",
            opacity: 0.15,
            width: 1,
          },
          move: {
            enable: true,
            speed: 0.6,
            direction: "none" as const,
            random: true,
            straight: false,
            outModes: { default: "bounce" as const },
            attract: { enable: true, rotateX: 600, rotateY: 1200 },
          },
          size: { value: { min: 1, max: 3 } },
          opacity: { value: { min: 0.1, max: 0.5 }, animation: { enable: true, speed: 0.5, sync: false } },
        },
        interactivity: {
          events: {
            onHover: { enable: true, mode: "grab" },
          },
          modes: {
            grab: {
              distance: 200,
              links: { opacity: 0.4, color: "#34d399" },
            },
          },
        },
        detectRetina: true,
        smooth: true,
      }}
    />
  );
}
