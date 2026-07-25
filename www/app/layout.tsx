import type { Metadata } from "next";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import { CustomCursor } from "../components/cursor";
import "../app/globals.css";

export const metadata: Metadata = {
  title: "OneInit — One command to init your dev machine",
  description: "The first tool to install on a new computer. AI-first environment initializer.",
  openGraph: { title: "OneInit", description: "One command to init your dev machine.", type: "website" },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className={`${GeistSans.variable} ${GeistMono.variable} font-sans antialiased bg-zinc-950 text-zinc-100`}>
        <CustomCursor />
        {children}
      </body>
    </html>
  );
}
