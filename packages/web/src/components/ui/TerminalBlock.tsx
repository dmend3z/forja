import { CopyButton } from "./CopyButton";

interface TerminalBlockProps {
  title?: string;
  children: React.ReactNode;
  copyText?: string;
}

export function TerminalBlock({ title = "Terminal", children, copyText }: TerminalBlockProps) {
  return (
    <div className="terminal-body rounded-lg border border-border overflow-hidden bg-bg-code">
      <div className="flex items-center justify-between px-4 py-2.5 bg-bg-terminal-header border-b border-border-terminal">
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5">
            <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
            <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
            <span className="w-3 h-3 rounded-full bg-[#28c840]" />
          </div>
          <span className="text-xs text-white/50 ml-2 font-mono">{title}</span>
        </div>
        {copyText && <CopyButton text={copyText} />}
      </div>
      <div className="p-4 font-mono text-sm leading-relaxed overflow-x-auto">
        {children}
      </div>
    </div>
  );
}
