"use client";

import { motion } from "motion/react";

interface TransitionOverlayProps {
  onComplete?: () => void;
}

export function TransitionOverlay({ onComplete }: TransitionOverlayProps) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.5 }}
      onAnimationComplete={() => {
        setTimeout(() => onComplete?.(), 2500);
      }}
      className="fixed inset-0 z-[100] flex items-center justify-center bg-[#0d0d0d]/95 backdrop-blur-sm"
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.3, duration: 0.6 }}
        className="text-center px-8"
      >
        <div className="border border-emerald-500/20 rounded-lg p-10 bg-[#141414]/90">
          <motion.p
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5, duration: 0.5 }}
            className="text-sm text-[#6a6a6a] tracking-widest uppercase mb-5"
          >
            What if...
          </motion.p>
          <motion.p
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.9, duration: 0.5 }}
            className="text-2xl md:text-3xl font-semibold text-[#d4d4d4] leading-relaxed"
          >
            one command
          </motion.p>
          <motion.p
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 1.3, duration: 0.5 }}
            className="text-2xl md:text-3xl font-bold bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent mt-1"
          >
            could set up everything?
          </motion.p>
          <motion.p
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 1.8, duration: 0.5 }}
            className="text-xs text-[#555] mt-5"
          >
            Python. PATH. Mirrors. All at once.
          </motion.p>
        </div>
      </motion.div>
    </motion.div>
  );
}
