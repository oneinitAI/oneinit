import type { Metadata } from "next";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import "../app/globals.css";

export const metadata: Metadata = {
  title: "OneInit - One command to init your dev machine",
  description:
    "The first tool to install on a new computer. Python, Node.js, Rust, Go - installed, mirrored, PATH-configured. All in one line.",
  openGraph: {
    title: "OneInit - One command to init your dev machine",
    description:
      "AI-first environment initializer. Install, configure, and migrate dev tools with one command.",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${GeistSans.variable} ${GeistMono.variable} font-sans antialiased bg-zinc-950 text-zinc-100 selection:bg-emerald-500/30`}
      >
        {children}
      </body>
    </html>
  );
}
