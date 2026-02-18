interface SectionHeaderProps {
  label: string;
  title: string;
  subtitle?: string;
}

export function SectionHeader({ label, title, subtitle }: SectionHeaderProps) {
  return (
    <div className="text-center mb-12">
      <span className="inline-block text-xs font-mono font-medium tracking-wider uppercase text-accent mb-3">
        {label}
      </span>
      <h2 className="text-3xl md:text-4xl font-semibold text-text mb-4">
        {title}
      </h2>
      {subtitle && (
        <p className="text-text-muted text-lg max-w-2xl mx-auto leading-relaxed">
          {subtitle}
        </p>
      )}
    </div>
  );
}
