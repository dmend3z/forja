import { SectionHeader } from "@/components/ui/SectionHeader";
import { Search, Download, Play, Layout } from "lucide-react";

export function DesktopSection() {
  return (
    <section id="desktop" className="py-20 md:py-28">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="05 | Desktop"
          title="Your skills marketplace, visual."
          subtitle="Browse, search, and install skills from a native desktop app. Run sparks, manage specs, and see execution in real time — no terminal required."
        />

        <div className="grid md:grid-cols-2 gap-8 items-center mt-12">
          <div className="space-y-4">
            {[
              { icon: Search, title: "Browse & search", desc: "Filter skills by phase, language, or keyword with instant results." },
              { icon: Download, title: "One-click install", desc: "Install and uninstall skills with a single click. No CLI needed." },
              { icon: Play, title: "Run sparks visually", desc: "Create specs, launch pipelines, and watch progress in real time." },
              { icon: Layout, title: "Native experience", desc: "Built with Tauri — fast, lightweight, runs on macOS, Windows, and Linux." },
            ].map((item) => (
              <div key={item.title} className="flex gap-4 p-4 rounded-lg border border-border bg-bg-card">
                <div className="flex-shrink-0 w-9 h-9 rounded-lg bg-desktop/15 flex items-center justify-center">
                  <item.icon size={18} className="text-desktop" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold text-text mb-1">{item.title}</h3>
                  <p className="text-xs text-text-muted leading-relaxed">{item.desc}</p>
                </div>
              </div>
            ))}
          </div>

          {/* App mockup placeholder */}
          <div className="rounded-xl border border-border bg-bg-card overflow-hidden">
            <div className="flex items-center gap-2 px-4 py-3 border-b border-border bg-bg-elevated">
              <div className="flex gap-1.5">
                <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
                <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
                <span className="w-3 h-3 rounded-full bg-[#28c840]" />
              </div>
              <span className="text-xs text-text-subtle ml-2">forja Desktop</span>
            </div>
            <div className="p-8 flex flex-col items-center justify-center min-h-[280px] text-center">
              <Layout size={48} className="text-border-strong mb-4" />
              <p className="text-sm text-text-subtle">Desktop app preview</p>
              <p className="text-xs text-text-subtle mt-1">Coming soon</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
