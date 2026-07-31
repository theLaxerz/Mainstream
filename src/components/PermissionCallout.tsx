import type { ReactNode } from "react";
import "./PermissionCallout.css";

type Props = {
  title: string;
  body: string;
  steps?: string[];
  actionLabel?: string;
  onAction?: () => void;
  children?: ReactNode;
};

export function PermissionCallout({
  title,
  body,
  steps,
  actionLabel,
  onAction,
  children,
}: Props) {
  return (
    <div className="permission-callout">
      <p className="permission-callout-title">{title}</p>
      <p className="module-empty">{body}</p>
      {steps && steps.length > 0 ? (
        <ol className="permission-callout-steps">
          {steps.map((step) => (
            <li key={step}>{step}</li>
          ))}
        </ol>
      ) : null}
      {children}
      {actionLabel && onAction ? (
        <button type="button" className="btn btn-primary" onClick={onAction}>
          {actionLabel}
        </button>
      ) : null}
    </div>
  );
}
