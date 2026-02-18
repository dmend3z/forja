import { CopyButton } from "@/components/ui/CopyButton";
import { ExternalLink, ArrowDown } from "lucide-react";

export function HeroSection() {
  return (
    <section id="hero" className="relative pt-32 pb-20 md:pt-40 md:pb-28 overflow-hidden">
      {/* Subtle grid background */}
      <div
        className="absolute inset-0 opacity-[0.06]"
        style={{
          backgroundImage: `linear-gradient(rgba(0,0,0,1) 1px, transparent 1px),
                            linear-gradient(90deg, rgba(0,0,0,1) 1px, transparent 1px)`,
          backgroundSize: "60px 60px",
        }}
      />

      <div className="relative max-w-4xl mx-auto px-4 text-center">
        <span className="inline-block text-xs font-mono text-text-muted border border-border rounded-full px-3 py-1 mb-6">
          v0.1.1 &mdash; Open Source &middot; MIT
        </span>

        <h1
          className="font-bold text-text mb-6"
          style={{ fontSize: "clamp(3rem, 8vw, 5.5rem)", lineHeight: 1.05 }}
        >
          Stop configuring,{" "}
          <span className="text-accent">start shipping</span>
        </h1>

        <p className="text-lg md:text-xl text-text-muted max-w-2xl mx-auto mb-10 leading-relaxed">
          Agent manager for Claude Code. 25 curated agents across 5 dev phases
          &mdash; Research, Code, Test, Review, Deploy.
        </p>

        <div className="terminal-body inline-flex items-center gap-3 border border-dashed border-border-strong rounded-lg px-4 py-3 font-mono text-sm mb-8 bg-bg-code">
          <span className="text-text-subtle">$</span>
          <span className="text-text">npm install -g forja-cli</span>
          <CopyButton text="npm install -g forja-cli" />
        </div>

        <div className="flex flex-wrap items-center justify-center gap-4">
          <a
            href="https://github.com/dmend3z/forja"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 text-sm font-medium text-white bg-accent px-5 py-2.5 rounded-full hover:bg-accent/90 transition-colors"
          >
            Source Code <ExternalLink size={14} />
          </a>
          <a
            href="https://github.com/dmend3z/forja"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 text-sm font-medium text-text border border-border px-5 py-2.5 rounded-full hover:bg-bg-elevated transition-colors"
          >
            Skills Registry <ExternalLink size={14} />
          </a>
          <a
            href="#quickstart"
            className="inline-flex items-center gap-2 text-sm font-medium text-text-muted hover:text-text transition-colors"
          >
            Get Started <ArrowDown size={14} />
          </a>
        </div>
      </div>
    </section>
  );
}
