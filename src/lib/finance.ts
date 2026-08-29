import { invoke } from "@tauri-apps/api/core";
import { openTarget } from "./api";

export type AccountKind =
  | "checking"
  | "savings"
  | "cash"
  | "credit"
  | "investment"
  | "other";

export type AccountWithBalance = {
  id: number;
  name: string;
  kind: AccountKind | string;
  currency: string;
  createdAt: string;
  balance: number;
};

export type Category = {
  id: number;
  name: string;
  color: string | null;
};

export type TransactionView = {
  id: number;
  accountId: number;
  accountName: string;
  categoryId: number | null;
  categoryName: string | null;
  amount: number;
  description: string | null;
  postedAt: string;
  createdAt: string;
  externalId: string | null;
};

export type FinanceDaySpend = {
  day: string;
  spent: number;
  income: number;
};

export type FinanceCategorySpend = {
  name: string;
  spent: number;
};

export type FinanceSummary = {
  cashTotal: number;
  netTotal: number;
  spentThisMonth: number;
  incomeThisMonth: number;
  accounts: AccountWithBalance[];
  recent: TransactionView[];
  spendByDay: FinanceDaySpend[];
  spendByCategory: FinanceCategorySpend[];
};

export type ImportCsvResult = {
  imported: number;
  skipped: number;
  format: string;
};

export type CsvFormat =
  | "auto"
  | "apple_card"
  | "chase"
  | "bofa"
  | "capital_one"
  | "citi"
  | "discover"
  | "generic";

export const BANK_LINKS: { label: string; url: string }[] = [
  { label: "Chase", url: "https://secure.chase.com" },
  { label: "Bank of America", url: "https://www.bankofamerica.com" },
  { label: "Capital One", url: "https://www.capitalone.com" },
  { label: "Citi", url: "https://www.citi.com" },
  { label: "Discover", url: "https://www.discover.com" },
  { label: "Amex", url: "https://www.americanexpress.com" },
  { label: "Apple Card", url: "https://card.apple.com" },
];

export function formatMoney(amount: number, currency = "USD"): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      maximumFractionDigits: 2,
    }).format(amount);
  } catch {
    return amount.toFixed(2);
  }
}

export function formatPosted(iso: string): string {
  try {
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

function pad(n: number) {
  return String(n).padStart(2, "0");
}

export function localDayKey(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function fillSpendDays(
  rows: FinanceDaySpend[],
  days = 14,
  now = new Date(),
): FinanceDaySpend[] {
  const byDay = new Map(rows.map((row) => [row.day.slice(0, 10), row]));
  const out: FinanceDaySpend[] = [];
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(now.getFullYear(), now.getMonth(), now.getDate() - i);
    const key = localDayKey(d);
    const row = byDay.get(key);
    out.push({
      day: key,
      spent: row?.spent ?? 0,
      income: row?.income ?? 0,
    });
  }
  return out;
}

export async function getFinanceSummary(): Promise<FinanceSummary> {
  return invoke("get_finance_summary");
}

export async function listAccounts(): Promise<AccountWithBalance[]> {
  return invoke("list_accounts");
}

export async function createAccount(input: {
  name: string;
  kind: AccountKind | string;
  currency?: string;
}): Promise<{ id: number; name: string; kind: string; currency: string; createdAt: string }> {
  return invoke("create_account", {
    input: {
      name: input.name,
      kind: input.kind,
      currency: input.currency ?? null,
    },
  });
}

export async function updateAccount(
  id: number,
  patch: { name?: string; kind?: string; currency?: string },
): Promise<{ id: number; name: string; kind: string; currency: string; createdAt: string }> {
  return invoke("update_account", { input: { id, ...patch } });
}

export async function deleteAccount(id: number): Promise<void> {
  return invoke("delete_account", { id });
}

export async function listTransactions(
  limit?: number,
  accountId?: number | null,
): Promise<TransactionView[]> {
  return invoke("list_transactions", {
    limit: limit ?? null,
    accountId: accountId ?? null,
  });
}

export async function createTransaction(input: {
  accountId: number;
  amount: number;
  description?: string;
  postedAt?: string;
  categoryId?: number | null;
}): Promise<unknown> {
  return invoke("create_transaction", {
    input: {
      accountId: input.accountId,
      amount: input.amount,
      description: input.description ?? null,
      postedAt: input.postedAt ?? null,
      categoryId: input.categoryId ?? null,
      externalId: null,
    },
  });
}

export async function deleteTransaction(id: number): Promise<void> {
  return invoke("delete_transaction", { id });
}

export async function importTransactionsCsv(input: {
  accountId: number;
  csvText: string;
  format?: CsvFormat | null;
}): Promise<ImportCsvResult> {
  return invoke("import_transactions_csv", {
    input: {
      accountId: input.accountId,
      csvText: input.csvText,
      format:
        input.format && input.format !== "auto" ? input.format : null,
    },
  });
}

export async function openBankSite(url: string): Promise<void> {
  await openTarget("url", url);
}
