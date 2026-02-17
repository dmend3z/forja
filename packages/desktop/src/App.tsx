import { BrowserRouter, Routes, Route } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ProjectsPage } from "./pages/ProjectsPage";
import { SparksPage } from "./pages/SparksPage";
import { SpecsPage } from "./pages/SpecsPage";
import { AutomationsPage } from "./pages/AutomationsPage";
import { MarketplacePage } from "./pages/MarketplacePage";
import { SkillDetailPage } from "./pages/SkillDetailPage";
import { SettingsPage } from "./pages/SettingsPage";

export default function App() {
  return (
    <BrowserRouter>
      <div className="flex h-screen bg-background text-foreground">
        <Sidebar />
        <main className="flex-1 overflow-auto">
          <Routes>
            <Route path="/" element={<ProjectsPage />} />
            <Route path="/project/:id/sparks" element={<SparksPage />} />
            <Route path="/project/:id/specs" element={<SpecsPage />} />
            <Route
              path="/project/:id/automations"
              element={<AutomationsPage />}
            />
            <Route path="/marketplace" element={<MarketplacePage />} />
            <Route
              path="/marketplace/:skillId"
              element={<SkillDetailPage />}
            />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}
