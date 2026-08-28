import { createContext, useContext, type ReactNode } from "react";
import type { ModuleRefreshResult } from "./types";

export type ModuleSyncEntry = ModuleRefreshResult & { at: string };

export type SyncState = {
  finishedAt: string | null;
  byModule: Record<string, ModuleSyncEntry>;
};

const empty: SyncState = { finishedAt: null, byModule: {} };

const SyncStateContext = createContext<SyncState>(empty);
const CurrentModuleIdContext = createContext<string | null>(null);

export function SyncStateProvider({
  value,
  children,
}: {
  value: SyncState;
  children: ReactNode;
}) {
  return (
    <SyncStateContext.Provider value={value}>
      {children}
    </SyncStateContext.Provider>
  );
}

export function ModuleSyncScope({
  id,
  children,
}: {
  id: string;
  children: ReactNode;
}) {
  return (
    <CurrentModuleIdContext.Provider value={id}>
      {children}
    </CurrentModuleIdContext.Provider>
  );
}

export function useSyncState(): SyncState {
  return useContext(SyncStateContext);
}

export function useCurrentModuleId(): string | null {
  return useContext(CurrentModuleIdContext);
}

export function useModuleSync(id?: string | null): ModuleSyncEntry | null {
  const state = useContext(SyncStateContext);
  const scoped = useContext(CurrentModuleIdContext);
  const key = id ?? scoped;
  if (!key) return null;
  return state.byModule[key] ?? null;
}

export function formatSyncedAgo(iso: string | null | undefined, now = Date.now()): string | null {
  if (!iso) return null;
  const then = Date.parse(iso);
  if (!Number.isFinite(then)) return null;
  const minutes = Math.max(0, Math.round((now - then) / 60_000));
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.round(hours / 24)}d ago`;
}

export function buildSyncState(
  finishedAt: string,
  modules: ModuleRefreshResult[],
): SyncState {
  const byModule: Record<string, ModuleSyncEntry> = {};
  for (const row of modules) {
    byModule[row.module] = { ...row, at: finishedAt };
  }
  return { finishedAt, byModule };
}
