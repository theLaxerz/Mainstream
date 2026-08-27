import type { CSSProperties, ReactNode } from "react";
import "./ModuleSection.css";

type Props = {
  title: string;
  eyebrow?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
  style?: CSSProperties;
  count?: number | null;
  accent?: "teal" | "accent" | "ink";
};

export function ModuleSkeleton({ rows = 3 }: { rows?: number }) {
  return (
    <div className="module-skeleton" aria-hidden="true">
      {Array.from({ length: rows }, (_, i) => (
        <div
          key={i}
          className="module-skeleton-row"
          style={{ animationDelay: `${i * 0.08}s` }}
        />
      ))}
    </div>
  );
}

export function ModuleSection({
  title,
  eyebrow,
  action,
  children,
  className = "",
  style,
  count,
  accent = "teal",
}: Props) {
  return (
    <section
      className={`module accent-${accent} ${className}`.trim()}
      style={style}
    >
      <header className="module-header">
        <div className="module-heading">
          {eyebrow ? <p className="module-eyebrow">{eyebrow}</p> : null}
          <div className="module-title-row">
            <h2 className="module-title">{title}</h2>
            {typeof count === "number" ? (
              <span className="module-count" aria-label={`${count} items`}>
                {count}
              </span>
            ) : null}
          </div>
        </div>
        {action ? <div className="module-action">{action}</div> : null}
      </header>
      <div className="module-body">{children}</div>
    </section>
  );
}
