/** Browser-only preview data so `npm run dev` still shows the dashboard look. */

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function previewCalendarEvents() {
  const today = new Date();
  const y = today.getFullYear();
  const m = String(today.getMonth() + 1).padStart(2, "0");
  const d = String(today.getDate()).padStart(2, "0");
  const tomorrow = new Date(today);
  tomorrow.setDate(today.getDate() + 1);
  const tY = tomorrow.getFullYear();
  const tM = String(tomorrow.getMonth() + 1).padStart(2, "0");
  const tD = String(tomorrow.getDate()).padStart(2, "0");

  return [
    {
      id: "preview-standup",
      title: "Standup",
      start: `${y}-${m}-${d}T13:30:00.000Z`,
      end: `${y}-${m}-${d}T14:00:00.000Z`,
      isAllDay: false,
      location: null,
      calendarName: "Work",
    },
    {
      id: "preview-dinner",
      title: "Dinner with Alex",
      start: `${tY}-${tM}-${tD}T23:00:00.000Z`,
      end: `${tY}-${tM}-${tD}T23:45:00.000Z`,
      isAllDay: false,
      location: "Home",
      calendarName: "Personal",
    },
  ];
}

export function previewFinanceSummary() {
  const today = new Date();
  const spendByDay = Array.from({ length: 14 }, (_, i) => {
    const d = new Date(today.getFullYear(), today.getMonth(), today.getDate() - (13 - i));
    const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
    const spent = [18, 0, 42, 12, 9, 0, 64, 22, 0, 31, 15, 8, 47, 21][i] ?? 0;
    return { day: key, spent, income: i === 10 ? 2400 : 0 };
  });
  const spentThisMonth = spendByDay.reduce((s, d) => s + d.spent, 0);
  return {
    cashTotal: 4280.12,
    netTotal: 3125.4,
    spentThisMonth,
    incomeThisMonth: 2400,
    accounts: [
      {
        id: 1,
        name: "Checking",
        kind: "checking",
        currency: "USD",
        createdAt: today.toISOString(),
        balance: 2480.12,
      },
      {
        id: 2,
        name: "Apple Card",
        kind: "credit",
        currency: "USD",
        createdAt: today.toISOString(),
        balance: -1154.72,
      },
    ],
    recent: [
      {
        id: 1,
        accountId: 2,
        accountName: "Apple Card",
        categoryId: 1,
        categoryName: "Dining",
        amount: -21,
        description: "Neighborhood cafe",
        postedAt: today.toISOString(),
        createdAt: today.toISOString(),
        externalId: null,
      },
    ],
    spendByDay,
    spendByCategory: [
      { name: "Dining", spent: 86 },
      { name: "Groceries", spent: 64 },
      { name: "Transit", spent: 31 },
    ],
  };
}
