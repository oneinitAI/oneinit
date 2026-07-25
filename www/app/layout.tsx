import type { Metadata } from "next";
import { InteractiveBg } from "../components/interactive-bg";
import "./globals.css";

export const metadata: Metadata = {
  title: "OneInit — One Command to Init Your Dev Machine",
  description: "AI-first environment initializer. Python, Node, Rust, Go — installed, mirrored, PATH-configured in one line.",
  openGraph: { title: "OneInit", description: "One command to init your dev machine.", type: "website" },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="font-sans antialiased text-zinc-200 bg-[#0a0a0f]">
        <InteractiveBg />
        <div className="relative z-10">{children}</div>
      </body>
    </html>
  );
}
