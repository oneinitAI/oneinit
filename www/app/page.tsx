import { Nav } from "../components/nav";
import { Hero } from "../components/hero";
import { Problem } from "../components/problem";
import { HorizontalScroll } from "../components/horizontal-scroll";
import { CommandShowcase } from "../components/command-showcase";
import { Capabilities } from "../components/capabilities";
import { Installation } from "../components/installation";
import { MarqueeStrip } from "../components/marquee-strip";
import { Footer } from "../components/footer";

export default function Home() {
  return (
    <main className="relative bg-zinc-950 text-zinc-100">
      <Nav />
      <Hero />
      <MarqueeStrip />
      <Problem />
      <HorizontalScroll />
      <CommandShowcase />
      <Capabilities />
      <Installation />
      <Footer />
    </main>
  );
}
