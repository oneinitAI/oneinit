"use client";
import { createContext, useContext, useEffect, useState } from "react";
import { dict, type Lang } from "@/lib/i18n";

type LangContextType = {
  lang: Lang;
  setLang: (l: Lang) => void;
  toggle: () => void;
  t: (key: string, vars?: Record<string, string | number>) => string;
};

const LangContext = createContext<LangContextType>({
  lang: "en",
  setLang: () => {},
  toggle: () => {},
  t: (k) => k,
});

export function LangProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>("en");

  // 记住用户选择
  useEffect(() => {
    const saved = localStorage.getItem("oneinit-lang") as Lang | null;
    if (saved === "en" || saved === "zh") setLangState(saved);
  }, []);

  const setLang = (l: Lang) => {
    setLangState(l);
    localStorage.setItem("oneinit-lang", l);
    document.documentElement.lang = l === "zh" ? "zh-CN" : "en";
  };

  const toggle = () => setLang(lang === "en" ? "zh" : "en");

  const t = (key: string, vars?: Record<string, string | number>) => {
    const entry = dict[key];
    let text = entry ? entry[lang] : key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        text = text.replace(`{${k}}`, String(v));
      }
    }
    return text;
  };

  return (
    <LangContext.Provider value={{ lang, setLang, toggle, t }}>
      {children}
    </LangContext.Provider>
  );
}

export function useLang() {
  return useContext(LangContext);
}
