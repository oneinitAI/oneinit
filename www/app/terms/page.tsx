"use client";
import { useLang } from "@/components/lang-provider";

export default function TermsPage() {
  const { t } = useLang();
  const sections = [
    { tk: "terms.s1t", dk: "terms.s1d" },
    { tk: "terms.s2t", dk: "terms.s2d" },
    { tk: "terms.s3t", dk: "terms.s3d" },
  ];

  return (
    <main className="relative min-h-screen bg-[#0a0a0f] text-zinc-200">
      <div className="mx-auto max-w-[720px] px-6 py-24">
        <a
          href="/"
          className="inline-flex items-center gap-2 font-mono text-sm text-emerald-500 hover:text-emerald-400 transition-colors"
        >
          ← {t("terms.back")}
        </a>

        <h1 className="mt-6 text-4xl font-bold tracking-tight text-white">
          {t("terms.title")}
        </h1>
        <p className="mt-2 font-mono text-xs text-zinc-600">{t("terms.updated")}</p>
        <p className="mt-6 text-zinc-400">{t("terms.p1")}</p>

        <div className="mt-10 space-y-8">
          {sections.map((s, i) => (
            <div key={i} className="rounded-2xl glass p-6">
              <div className="flex items-center gap-3">
                <span className="font-mono text-sm text-emerald-600">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <h2 className="text-lg font-bold text-white">{t(s.tk)}</h2>
              </div>
              <p className="mt-3 leading-relaxed text-zinc-400">{t(s.dk)}</p>
            </div>
          ))}
        </div>

        <p className="mt-12 font-mono text-xs text-zinc-700">
          OneInit · GPL-3.0 · {t("terms.updated")}
        </p>
      </div>
    </main>
  );
}
