import { getSetting, setSetting } from "./api";

export const MODULE_IDS = [
  "messages",
  "calendar",
  "email",
  "mail",
  "news",
  "finance",
  "notes",
  "health",
  "home",
  "youtube",
  "streaming",
  "shortcuts",
] as const;

export type ModuleId = (typeof MODULE_IDS)[number];

export type ModulePlacement = "auto" | "left" | "right" | "full";

export type ModuleLayoutEntry = {
  id: ModuleId;
  enabled: boolean;
  listLimit: number;
  placement: ModulePlacement;
  order: number;
};

export type DashboardLayout = {
  version: number;
  modules: ModuleLayoutEntry[];
};

const LAYOUT_VERSION = 2;

export const MODULE_META: Record<
  ModuleId,
  { title: string; eyebrow: string; blurb: string }
> = {
  messages: {
    title: "Messages",
    eyebrow: "Unread",
    blurb: "iMessage conversations grouped by chat",
  },
  calendar: {
    title: "Calendar",
    eyebrow: "Upcoming",
    blurb: "Events from macOS Calendar.app",
  },
  email: {
    title: "Email",
    eyebrow: "Important",
    blurb: "Google, Microsoft, or IMAP with importance filtering",
  },
  mail: {
    title: "Mail",
    eyebrow: "Physical",
    blurb: "USPS Informed Delivery envelope scans",
  },
  news: {
    title: "News",
    eyebrow: "Tailored",
    blurb: "Ranked RSS stories that learn from you",
  },
  finance: {
    title: "Finance",
    eyebrow: "Ledger",
    blurb: "Local balances, transactions, CSV import",
  },
  notes: {
    title: "Notes",
    eyebrow: "Recent",
    blurb: "Quick capture sorted by update time",
  },
  health: {
    title: "Health",
    eyebrow: "Apple Health",
    blurb: "Steps, sleep, and heart rate imports",
  },
  home: {
    title: "Home",
    eyebrow: "Cameras",
    blurb: "Ring status and Blink camera stills",
  },
  youtube: {
    title: "YouTube",
    eyebrow: "Channels",
    blurb: "Latest uploads from followed channels",
  },
  streaming: {
    title: "Streaming",
    eyebrow: "Watch",
    blurb: "What's hot across your services",
  },
  shortcuts: {
    title: "Shortcuts",
    eyebrow: "Launch",
    blurb: "Website and app launchers",
  },
};

const SETTING_KEY = "dashboard.layout.v1";

export const DEFAULT_MODULE_LIMITS: Record<ModuleId, number> = {
  messages: 10,
  calendar: 8,
  email: 10,
  mail: 12,
  news: 8,
  finance: 10,
  notes: 10,
  health: 7,
  home: 8,
  youtube: 10,
  streaming: 8,
  shortcuts: 12,
};

const DEFAULT_ORDER: ModuleId[] = [
  "messages",
  "calendar",
  "email",
  "mail",
  "news",
  "finance",
  "notes",
  "health",
  "home",
  "youtube",
  "streaming",
  "shortcuts",
];

const DEFAULT_PLACEMENT: Partial<Record<ModuleId, ModulePlacement>> = {
  mail: "full",
  streaming: "full",
};

export function defaultLayout(): DashboardLayout {
  return {
    version: LAYOUT_VERSION,
    modules: DEFAULT_ORDER.map((id, order) => ({
      id,
      enabled: true,
      listLimit: DEFAULT_MODULE_LIMITS[id],
      placement: DEFAULT_PLACEMENT[id] ?? "auto",
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

function parsePlacement(
  value: unknown,
  id: ModuleId,
  rawVersion: number,
): ModulePlacement {
  const placementRaw = typeof value === "string" ? value : "";
  let placement: ModulePlacement =
    placementRaw === "auto" ||
    placementRaw === "left" ||
    placementRaw === "right" ||
    placementRaw === "full"
      ? placementRaw
      : (DEFAULT_PLACEMENT[id] ?? "auto");
  // v1 stored auto-flow as "left" (labeled Auto in the UI).
  if (rawVersion < 2 && placement === "left") {
    placement = "auto";
  }
  return placement;
}

export function normalizeLayout(raw: unknown): DashboardLayout {
  const base = defaultLayout();
  if (!raw || typeof raw !== "object") return base;
  const rawObj = raw as { version?: unknown; modules?: unknown };
  const rawVersion = typeof rawObj.version === "number" ? rawObj.version : 1;
  const modulesRaw = rawObj.modules;
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
    const placement = parsePlacement(
      (item as { placement?: string }).placement,
      id,
      rawVersion,
    );
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
  return { version: LAYOUT_VERSION, modules };
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

export async function saveDashboardLayout(
  layout: DashboardLayout,
): Promise<void> {
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

export type GridStyle = {
  gridColumn?: string;
};

export function placementGridStyle(
  placement: ModulePlacement,
): GridStyle | undefined {
  if (placement === "full") {
    return { gridColumn: "1 / -1" };
  }
  if (placement === "right") {
    return { gridColumn: "2 / 3" };
  }
  if (placement === "left") {
    return { gridColumn: "1 / 2" };
  }
  return undefined;
}

export function moveModule(
  layout: DashboardLayout,
  id: ModuleId,
  direction: -1 | 1,
): DashboardLayout {
  const modules = [...layout.modules].sort((a, b) => a.order - b.order);
  const index = modules.findIndex((m) => m.id === id);
  if (index < 0) return layout;
  const target = index + direction;
  if (target < 0 || target >= modules.length) return layout;
  const next = [...modules];
  const [item] = next.splice(index, 1);
  next.splice(target, 0, item);
  return {
    version: layout.version ?? 1,
    modules: next.map((m, order) => ({ ...m, order })),
  };
}

export function updateModule(
  layout: DashboardLayout,
  id: ModuleId,
  patch: Partial<Omit<ModuleLayoutEntry, "id" | "order">>,
): DashboardLayout {
  return {
    version: layout.version ?? 1,
    modules: layout.modules.map((m) => {
      if (m.id !== id) return m;
      const listLimit =
        patch.listLimit !== undefined
          ? clampLimit(patch.listLimit, id)
          : m.listLimit;
      return { ...m, ...patch, listLimit };
    }),
  };
}

export function enabledModules(layout: DashboardLayout): ModuleLayoutEntry[] {
  return [...layout.modules]
    .filter((m) => m.enabled)
    .sort((a, b) => a.order - b.order);
}
