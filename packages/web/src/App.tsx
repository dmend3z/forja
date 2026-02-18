import { Nav } from "@/components/layout/Nav";
import { Footer } from "@/components/layout/Footer";
import { HeroSection } from "@/components/sections/HeroSection";
import { WhatsNewBanner } from "@/components/sections/WhatsNewBanner";
import { QuickStartSection } from "@/components/sections/QuickStartSection";
import { WhySection } from "@/components/sections/WhySection";
import { SparksSection } from "@/components/sections/SparksSection";
import { ScanSection } from "@/components/sections/ScanSection";
import { DesktopSection } from "@/components/sections/DesktopSection";
import { MonitorSection } from "@/components/sections/MonitorSection";
import { PhasesSection } from "@/components/sections/PhasesSection";
import { CatalogSection } from "@/components/sections/CatalogSection";
import { CommandsSection } from "@/components/sections/CommandsSection";
import { TeamsSection } from "@/components/sections/TeamsSection";
import { ProfilesSection } from "@/components/sections/ProfilesSection";
import { ArchitectureSection } from "@/components/sections/ArchitectureSection";

export function App() {
  return (
    <>
      <Nav />
      <main>
        <HeroSection />
        <WhatsNewBanner />
        <QuickStartSection />
        <WhySection />
        <SparksSection />
        <ScanSection />
        <DesktopSection />
        <MonitorSection />
        <PhasesSection />
        <CatalogSection />
        <CommandsSection />
        <TeamsSection />
        <ProfilesSection />
        <ArchitectureSection />
      </main>
      <Footer />
    </>
  );
}
