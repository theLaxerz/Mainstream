import type { CSSProperties, ReactNode } from "react";
import "./ModuleSection.css";

type Props = {
  title: string;
  eyebrow?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
  style?: CSSProperties;
};

export function ModuleSection({
  title,
  eyebrow,
  action,
  children,
  className = "",
  style,
}: Props) {
  return (
    <section className={`module ${className}`.trim()} style={style}>
      <header className="module-header">
        <div>
          {eyebrow ? <p className="module-eyebrow">{eyebrow}</p> : null}
          <h2 className="module-title">{title}</h2>
        </div>
        {action ? <div className="module-action">{action}</div> : null}
      </header>
      <div className="module-body">{children}</div>
    </section>
  );
}
