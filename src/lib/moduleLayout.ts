import { getSetting, setSetting } from "./api";

export const MODULE_IDS = [
  "messages",
  "email",
  "news",
  "finance",
  "notes",
  "shortcuts",
] as const;

export type ModuleId = (typeof MODULE_IDS)[number];

export type ModulePlacement = "left" | "right" | "full";

export type ModuleLayoutEntry = {
  id: ModuleId;
  enabled: boolean;
  listLimit: number;
  placement: ModulePlacement;
  order: number;
};

export type DashboardLayout = {
  modules: ModuleLayoutEntry[];
};

const SETTING_KEY = "dashboard.layout.v1";

export const DEFAULT_MODULE_LIMITS: Record<ModuleId, number> = {
  messages: 10,
  email: 10,
  news: 8,
  finance: 5,
  notes: 10,
  shortcuts: 12,
};

const DEFAULT_ORDER: ModuleId[] = [
  "messages",
  "email",
  "news",
  "finance",
  "notes",
  "shortcuts",
];

export function defaultLayout(): DashboardLayout {
  return {
    modules: DEFAULT_ORDER.map((id, order) => ({
      id,
      enabled: true,
      listLimit: DEFAULT_MODULE_LIMITS[id],
      placement: "left" as ModulePlacement,
      order,
    })),
  };
}

function isModuleId(value: string): value is ModuleId {
  return (MODULE_IDS as readonly string[]).includes(value);
}

function clampLimit(n: number, id: ModuleId): number {
  const min = 3;
  const max = id === "shortcuts" ? 24 : 50;
  if (!Number.isFinite(n)) return DEFAULT_MODULE_LIMITS[id];
  return Math.min(max, Math.max(min, Math.round(n)));
}

export function normalizeLayout(raw: unknown): DashboardLayout {
  const base = defaultLayout();
  if (!raw || typeof raw !== "object") return base;
  const modulesRaw = (raw as { modules?: unknown }).modules;
  if (!Array.isArray(modulesRaw)) return base;

  const byId = new Map<ModuleId, ModuleLayoutEntry>();
  for (const item of modulesRaw) {
    if (!item || typeof item !== "object") continue;
    const id = (item as { id?: string }).id;
    if (!id || !isModuleId(id)) continue;
    const enabled =
      typeof (item as { enabled?: boolean }).enabled === "boolean"
        ? (item as { enabled: boolean }).enabled
        : true;
    const listLimit = clampLimit(
      Number((item as { listLimit?: number }).listLimit),
      id,
    );
    const placementRaw = (item as { placement?: string }).placement;
    const placement: ModulePlacement =
      placementRaw === "right" || placementRaw === "full"
        ? placementRaw
        : "left";
    const order =
      typeof (item as { order?: number }).order === "number"
        ? (item as { order: number }).order
        : byId.size;
    byId.set(id, { id, enabled, listLimit, placement, order });
  }

  for (const entry of base.modules) {
    if (!byId.has(entry.id)) {
      byId.set(entry.id, entry);
    }
  }

  const modules = [...byId.values()].sort((a, b) => a.order - b.order);
  modules.forEach((m, i) => {
    m.order = i;
  });
  return { modules };
}

export async function loadDashboardLayout(): Promise<DashboardLayout> {
  try {
    const json = await getSetting(SETTING_KEY);
    if (!json) return defaultLayout();
    return normalizeLayout(JSON.parse(json));
  } catch {
    return defaultLayout();
  }
}

export async function saveDashboardLayout(layout: DashboardLayout): Promise<void> {
  const normalized = normalizeLayout(layout);
  await setSetting(SETTING_KEY, JSON.stringify(normalized));
}

export function getModuleEntry(
  layout: DashboardLayout,
  id: ModuleId,
): ModuleLayoutEntry {
  return (
    layout.modules.find((m) => m.id === id) ??
    defaultLayout().modules.find((m) => m.id === id)!
  );
}

export function placementGridStyle(
  placement: ModulePlacement,
): React.CSSProperties | undefined {
  if (placement === "full") {
    return { gridColumn: "1 / -1" };
  }
  if (placement === "right") {
    return { gridColumn: "2 / 3" };
  }
  return undefined;
}

// React.CSSProperties without importing React in a .ts file — use inline type
type GridStyle = {
  gridColumn?: string;
};

export function placementGridStylePlain(
  placement: ModulePlacement,
): GridStyle | undefined {
  if (placement === "full") {
    return { gridColumn: "1 / -1" };
  }
  if (placement === "right") {
    return { gridColumn: "2 / 3" };
  }
  return undefined;
}
