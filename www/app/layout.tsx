import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://oneinit.vercel.app"),
  title: "OneInit — One Command to Init Your Dev Machine",
  description: "AI-first environment initializer. Python, Node, Rust, Go — installed, mirrored, PATH-configured in one line.",
  icons: { icon: "/logo.png", apple: "/logo.png" },
  openGraph: { title: "OneInit", description: "One command to init your dev machine.", type: "website", images: ["/logo.png"] },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="font-sans antialiased text-zinc-200">
        {/* Pure CSS background — always visible, zero JS */}
        <div className="bg-grid" />
        <div className="bg-orb bg-orb-1" />
        <div className="bg-orb bg-orb-2" />
        <div className="bg-orb bg-orb-3" />
        <div className="bg-scanline" />
        <div className="relative z-10">{children}</div>
      </body>
    </html>
  );
}
