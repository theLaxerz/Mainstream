/** Broadcast so every dashboard module can reload without prop drilling. */
export const REFRESH_EVENT = "mainstream:refresh";

export function requestDashboardRefresh() {
  window.dispatchEvent(new CustomEvent(REFRESH_EVENT));
}

export function onDashboardRefresh(handler: () => void): () => void {
  const listener = () => handler();
  window.addEventListener(REFRESH_EVENT, listener);
  return () => window.removeEventListener(REFRESH_EVENT, listener);
}
