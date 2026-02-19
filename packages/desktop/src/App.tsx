import { BrowserRouter, Routes, Route } from "react-router";
import { Sidebar } from "./components/Sidebar";
import { ProjectsPage } from "./pages/ProjectsPage";
import { DashboardPage } from "./pages/DashboardPage";
import { TracksPage } from "./pages/TracksPage";
import { SpecsPage } from "./pages/SpecsPage";
import { PlansPage } from "./pages/PlansPage";
import { DecisionsPage } from "./pages/DecisionsPage";
import { RunsPage } from "./pages/RunsPage";
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
            <Route path="/project/:id/dashboard" element={<DashboardPage />} />
            <Route path="/project/:id/tracks" element={<TracksPage />} />
            <Route path="/project/:id/specs" element={<SpecsPage />} />
            <Route path="/project/:id/plans" element={<PlansPage />} />
            <Route path="/project/:id/decisions" element={<DecisionsPage />} />
            <Route path="/project/:id/runs" element={<RunsPage />} />
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
