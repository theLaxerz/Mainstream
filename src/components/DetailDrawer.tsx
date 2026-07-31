import type { ReactNode } from "react";
import "./DetailDrawer.css";

type Props = {
  open: boolean;
  title: string;
  eyebrow?: string;
  onClose: () => void;
  children: ReactNode;
  label?: string;
};

export function DetailDrawer({
  open,
  title,
  eyebrow,
  onClose,
  children,
  label,
}: Props) {
  if (!open) return null;

  return (
    <div
      className="detail-drawer-root"
      role="dialog"
      aria-modal="true"
      aria-label={label ?? title}
    >
      <button
        type="button"
        className="detail-drawer-backdrop"
        aria-label="Close"
        onClick={onClose}
      />
      <aside className="detail-drawer">
        <header className="detail-drawer-header">
          <div>
            {eyebrow ? <p className="module-eyebrow">{eyebrow}</p> : null}
            <h2 className="module-title">{title}</h2>
          </div>
          <button type="button" className="btn btn-ghost" onClick={onClose}>
            Close
          </button>
        </header>
        <div className="detail-drawer-body">{children}</div>
      </aside>
    </div>
  );
}
