import { Nav } from "../components/nav";
import { Hero } from "../components/hero";
import { Problem } from "../components/problem";
import { CommandShowcase } from "../components/command-showcase";
import { Capabilities } from "../components/capabilities";
import { Installation } from "../components/installation";
import { Stats } from "../components/stats";
import { Footer } from "../components/footer";
import { ParticleBackground } from "../components/particle-background";

export default function Home() {
  return (
    <main className="relative min-h-screen bg-zinc-950 text-zinc-100">
      <ParticleBackground />
      <Nav />
      <Hero />
      <Problem />
      <CommandShowcase />
      <Capabilities />
      <Installation />
      <Stats />
      <Footer />
    </main>
  );
}
