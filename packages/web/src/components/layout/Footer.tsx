import { Hammer } from "lucide-react";

export function Footer() {
  return (
    <footer className="border-t border-border bg-bg-subtle">
      <div className="max-w-6xl mx-auto px-4 py-12">
        <div className="flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="flex items-center gap-2">
            <Hammer size={18} className="text-accent" />
            <span className="font-semibold text-text">forja</span>
            <span className="text-text-subtle text-sm ml-2">Agent manager for Claude Code</span>
          </div>
          <ul className="flex items-center gap-6">
            <li>
              <a
                href="https://github.com/dmend3z/forja"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-text-muted hover:text-text transition-colors"
              >
                GitHub
              </a>
            </li>
            <li>
              <a
                href="https://github.com/dmend3z/forja"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-text-muted hover:text-text transition-colors"
              >
                Registry
              </a>
            </li>
            <li>
              <a
                href="https://github.com/dmend3z/forja/blob/main/LICENSE"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-text-muted hover:text-text transition-colors"
              >
                MIT License
              </a>
            </li>
          </ul>
        </div>
        <div className="text-center mt-8 text-sm text-text-subtle">
          Built by{" "}
          <a
            href="https://github.com/dmend3z"
            target="_blank"
            rel="noopener noreferrer"
            className="text-text-muted hover:text-text transition-colors"
          >
            Daniel Mendes
          </a>
        </div>
      </div>
    </footer>
  );
}
