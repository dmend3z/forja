import { SectionHeader } from "@/components/ui/SectionHeader";
import { TerminalBlock } from "@/components/ui/TerminalBlock";

export function ScanSection() {
  return (
    <section id="scan" className="py-20 md:py-28 bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4">
        <SectionHeader
          label="04 | Scan"
          title="Know your stack before you start."
          subtitle="forja scan analyzes your project — detects Rust, TypeScript, Python, Go, frameworks, ORMs — and recommends exactly which skills to install. AI deep-dive optional."
        />

        <div className="grid md:grid-cols-2 gap-8 items-start mt-12">
          <div>
            <TerminalBlock title="forja scan" copyText="forja scan">
              <div className="space-y-1">
                <p><span className="text-text-subtle">$ </span><span className="text-text">forja scan</span></p>
                <p className="text-text-subtle mt-3">Scanning project...</p>
                <p className="mt-2">
                  <span className="text-scan">{'>'}</span>{" "}
                  <span className="text-text">Language:</span>{" "}
                  <span className="text-text-muted">Rust</span>{" "}
                  <span className="text-monitor">98%</span>
                </p>
                <p>
                  <span className="text-scan">{'>'}</span>{" "}
                  <span className="text-text">Language:</span>{" "}
                  <span className="text-text-muted">TypeScript</span>{" "}
                  <span className="text-monitor">85%</span>
                </p>
                <p>
                  <span className="text-scan">{'>'}</span>{" "}
                  <span className="text-text">Framework:</span>{" "}
                  <span className="text-text-muted">React 19</span>{" "}
                  <span className="text-monitor">92%</span>
                </p>
                <p>
                  <span className="text-scan">{'>'}</span>{" "}
                  <span className="text-text">Build:</span>{" "}
                  <span className="text-text-muted">Vite 6</span>{" "}
                  <span className="text-monitor">90%</span>
                </p>
                <p className="mt-3 text-text-muted">Recommended skills:</p>
                <p>
                  <span className="text-accent">+</span>{" "}
                  <span className="text-text">code/rust/feature</span>
                </p>
                <p>
                  <span className="text-accent">+</span>{" "}
                  <span className="text-text">code/typescript/feature</span>
                </p>
                <p>
                  <span className="text-accent">+</span>{" "}
                  <span className="text-text">test/tdd/workflow</span>
                </p>
              </div>
            </TerminalBlock>
          </div>

          <div className="space-y-4">
            {[
              { title: "Auto-detect", desc: "Identifies languages, frameworks, ORMs, and build tools from your project files." },
              { title: "Confidence scores", desc: "Each detection comes with a confidence percentage — no guesswork." },
              { title: "Smart recommendations", desc: "Maps detected stack to the exact skills you need, ready to install." },
              { title: "AI deep-dive", desc: "Optional AI analysis for nuanced recommendations beyond file detection." },
            ].map((item) => (
              <div key={item.title} className="p-4 rounded-lg border border-border bg-bg-card">
                <h3 className="text-sm font-semibold text-text mb-1">{item.title}</h3>
                <p className="text-xs text-text-muted leading-relaxed">{item.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
