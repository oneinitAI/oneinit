"use client";
import { useEffect } from "react";
import AOS from "aos";
import { Nav } from "../components/nav";
import { Hero } from "../components/hero";
import { Stats } from "../components/stats";
import { Features } from "../components/features";
import { InstallBar } from "../components/install-bar";
import { Footer } from "../components/footer";

export default function Home() {
  useEffect(() => { AOS.init({ duration: 800, once: true }); }, []);
  return (
    <main className="relative min-h-screen">
      <Nav />
      <Hero />
      <Stats />
      <Features />
      <InstallBar />
      <Footer />
    </main>
  );
}
