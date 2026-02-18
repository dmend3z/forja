import { useState } from "react";
import { Clipboard, Check } from "lucide-react";

export function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  };

  return (
    <button
      onClick={handleCopy}
      aria-label="Copy to clipboard"
      className="text-text-subtle hover:text-text-muted transition-colors cursor-pointer"
    >
      {copied ? <Check size={16} className="text-monitor" /> : <Clipboard size={16} />}
    </button>
  );
}
