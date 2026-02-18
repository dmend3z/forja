import { useState } from "react";
import { Hammer, Menu, X } from "lucide-react";

const links = [
  { label: "Quick Start", href: "#quickstart" },
  { label: "Features", href: "#sparks" },
  { label: "Phases", href: "#phases" },
  { label: "Catalog", href: "#catalog" },
  { label: "CLI", href: "#commands" },
  { label: "Teams", href: "#teams" },
];

export function Nav() {
  const [open, setOpen] = useState(false);

  return (
    <header className="fixed top-0 left-0 right-0 z-50 bg-bg/80 backdrop-blur-md border-b border-dashed border-border">
      <nav className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
        <a href="#" className="flex items-center gap-2 text-text font-semibold text-lg">
          <Hammer size={20} className="text-accent" />
          forja
        </a>

        {/* Desktop links */}
        <ul className="hidden md:flex items-center gap-6">
          {links.map((link) => (
            <li key={link.href}>
              <a
                href={link.href}
                className="text-sm text-text-muted hover:text-text transition-colors"
              >
                {link.label}
              </a>
            </li>
          ))}
          <li>
            <a
              href="https://github.com/dmend3z/forja"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm font-medium text-white bg-accent px-3 py-1.5 rounded-full hover:bg-accent/90 transition-colors"
            >
              GitHub
            </a>
          </li>
        </ul>

        {/* Mobile toggle */}
        <button
          onClick={() => setOpen(!open)}
          className="md:hidden text-text-muted hover:text-text"
          aria-label="Toggle navigation"
          aria-expanded={open}
        >
          {open ? <X size={24} /> : <Menu size={24} />}
        </button>
      </nav>

      {/* Mobile menu */}
      {open && (
        <div className="md:hidden border-t border-dashed border-border bg-bg">
          <ul className="flex flex-col py-4 px-4 gap-1">
            {links.map((link) => (
              <li key={link.href}>
                <a
                  href={link.href}
                  onClick={() => setOpen(false)}
                  className="block py-2 text-sm text-text-muted hover:text-text transition-colors"
                >
                  {link.label}
                </a>
              </li>
            ))}
            <li>
              <a
                href="https://github.com/dmend3z/forja"
                target="_blank"
                rel="noopener noreferrer"
                onClick={() => setOpen(false)}
                className="block py-2 text-sm text-accent hover:text-text transition-colors"
              >
                GitHub
              </a>
            </li>
          </ul>
        </div>
      )}
    </header>
  );
}
