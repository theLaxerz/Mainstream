import { useEffect, type ReactNode } from "react";
import "./DetailDrawer.css";

type Props = {
  open: boolean;
  title: string;
  eyebrow?: string;
  onClose: () => void;
  children: ReactNode;
  label?: string;
  wide?: boolean;
};

export function DetailDrawer({
  open,
  title,
  eyebrow,
  onClose,
  children,
  label,
  wide,
}: Props) {
  useEffect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      document.body.style.overflow = prev;
      window.removeEventListener("keydown", onKey);
    };
  }, [open, onClose]);

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
      <aside className={`detail-drawer ${wide ? "is-wide" : ""}`}>
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
