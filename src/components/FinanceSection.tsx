import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  BANK_LINKS,
  createAccount,
  createTransaction,
  deleteAccount,
  deleteTransaction,
  formatMoney,
  formatPosted,
  getFinanceSummary,
  importTransactionsCsv,
  listTransactions,
  openBankSite,
  type AccountKind,
  type AccountWithBalance,
  type CsvFormat,
  type FinanceSummary,
  type TransactionView,
} from "../lib/finance";
import { onDashboardRefresh } from "../lib/refresh";
import { DetailDrawer } from "./DetailDrawer";
import { ModuleSection } from "./ModuleSection";

const ACCOUNT_KINDS: AccountKind[] = [
  "checking",
  "savings",
  "cash",
  "credit",
  "investment",
  "other",
];

type Props = { limit?: number };

export function FinanceSection({ limit = 10 }: Props) {
  const [summary, setSummary] = useState<FinanceSummary | null>(null);
  const [transactions, setTransactions] = useState<TransactionView[]>([]);
  const [manageOpen, setManageOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const [acctName, setAcctName] = useState("");
  const [acctKind, setAcctKind] = useState<AccountKind>("checking");

  const [txnAccountId, setTxnAccountId] = useState<number | "">("");
  const [txnAmount, setTxnAmount] = useState("");
  const [txnDesc, setTxnDesc] = useState("");
  const [txnDate, setTxnDate] = useState(() =>
    new Date().toISOString().slice(0, 10),
  );

  const [importAccountId, setImportAccountId] = useState<number | "">("");
  const [importFormat, setImportFormat] = useState<CsvFormat>("auto");
  const fileRef = useRef<HTMLInputElement>(null);

  async function refresh() {
    try {
      const next = await getFinanceSummary();
      setSummary(next);
      if (manageOpen) {
        const rows = await listTransactions(50);
        setTransactions(rows);
      } else {
        setTransactions(next.recent);
      }
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    return onDashboardRefresh(() => void refresh());
  }, [manageOpen]);

  const accounts: AccountWithBalance[] = summary?.accounts ?? [];

  async function onCreateAccount(e: FormEvent) {
    e.preventDefault();
    if (!acctName.trim()) return;
    try {
      await createAccount({ name: acctName.trim(), kind: acctKind });
      setAcctName("");
      setStatus("Account created.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onDeleteAccount(id: number) {
    try {
      await deleteAccount(id);
      setStatus("Account deleted.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onCreateTxn(e: FormEvent) {
    e.preventDefault();
    if (txnAccountId === "" || !txnAmount.trim()) return;
    const amount = Number(txnAmount);
    if (Number.isNaN(amount)) {
      setError("Amount must be a number.");
      return;
    }
    try {
      await createTransaction({
        accountId: txnAccountId,
        amount,
        description: txnDesc.trim() || undefined,
        postedAt: txnDate ? `${txnDate}T12:00:00Z` : undefined,
      });
      setTxnAmount("");
      setTxnDesc("");
      setStatus("Transaction added.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onDeleteTxn(id: number) {
    try {
      await deleteTransaction(id);
      setStatus("Transaction deleted.");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function onImportFile(file: File | null) {
    if (!file || importAccountId === "") {
      setError("Choose an account and a CSV file.");
      return;
    }
    try {
      const csvText = await file.text();
      const result = await importTransactionsCsv({
        accountId: importAccountId,
        csvText,
        format: importFormat,
      });
      setStatus(
        `Imported ${result.imported} (${result.format}); skipped ${result.skipped} duplicates.`,
      );
      if (fileRef.current) fileRef.current.value = "";
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  function bankLinks() {
    return (
      <div className="finance-bank-links">
        {BANK_LINKS.map((link) => (
          <button
            key={link.url}
            type="button"
            className="btn btn-ghost btn-icon"
            onClick={() =>
              void openBankSite(link.url).catch((err) => {
                setError(err instanceof Error ? err.message : String(err));
              })
            }
          >
            {link.label}
          </button>
        ))}
      </div>
    );
  }

  return (
    <>
      <ModuleSection
        title="Finance"
        eyebrow="Snapshot"
        action={
          <div className="row-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => setManageOpen(true)}
            >
              Manage
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void refresh()}
            >
              Refresh
            </button>
          </div>
        }
        count={!loading ? transactions.length : null}
        accent="accent"
      >
        {error && !manageOpen ? <p className="module-empty">{error}</p> : null}
        {status && !manageOpen ? <p className="module-empty">{status}</p> : null}
        {loading ? <p className="module-empty">Loading finance…</p> : null}

        {!loading ? (
          <>
            <div className="finance-totals">
              <div>
                <p className="module-row-meta">Cash</p>
                <p className="module-row-title finance-total">
                  {formatMoney(summary?.cashTotal ?? 0)}
                </p>
              </div>
              <div>
                <p className="module-row-meta">Net</p>
                <p className="module-row-title finance-total">
                  {formatMoney(summary?.netTotal ?? 0)}
                </p>
              </div>
            </div>

            {accounts.length > 0 ? (
              <ul className="module-list finance-accounts-compact">
                {accounts.slice(0, 4).map((acct) => (
                  <li key={acct.id}>
                    <div className="module-row-main">
                      <p className="module-row-title">{acct.name}</p>
                      <p className="module-row-meta">{acct.kind}</p>
                    </div>
                    <p
                      className={`finance-amount ${acct.balance < 0 ? "is-neg" : ""}`}
                    >
                      {formatMoney(acct.balance, acct.currency)}
                    </p>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="module-empty">
                No accounts yet — open Manage to add one or import a CSV.
              </p>
            )}

            <p className="module-eyebrow finance-subhead">Recent</p>
            {transactions.length === 0 ? (
              <p className="module-empty">No transactions yet.</p>
            ) : (
              <ul className="module-list">
                {transactions.slice(0, limit).map((txn) => (
                  <li key={txn.id}>
                    <div className="module-row-main">
                      <p className="module-row-title">
                        {txn.description || "Transaction"}
                      </p>
                      <p className="module-row-meta">
                        {txn.accountName} · {formatPosted(txn.postedAt)}
                      </p>
                    </div>
                    <p
                      className={`finance-amount ${txn.amount < 0 ? "is-neg" : ""}`}
                    >
                      {formatMoney(txn.amount)}
                    </p>
                  </li>
                ))}
              </ul>
            )}

            {bankLinks()}
          </>
        ) : null}
      </ModuleSection>

      <DetailDrawer
        open={manageOpen}
        title="Finance ledger"
        eyebrow="Manage"
        onClose={() => setManageOpen(false)}
      >
        {error ? <p className="module-empty">{error}</p> : null}
        {status ? <p className="module-empty">{status}</p> : null}

        <form onSubmit={onCreateAccount}>
          <p className="module-eyebrow finance-subhead">Accounts</p>
          <div className="field-row">
            <input
              className="field"
              value={acctName}
              onChange={(e) => setAcctName(e.target.value)}
              placeholder="Account name"
              aria-label="Account name"
            />
            <select
              className="field-select"
              value={acctKind}
              onChange={(e) => setAcctKind(e.target.value as AccountKind)}
              aria-label="Account kind"
            >
              {ACCOUNT_KINDS.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
            <button type="submit" className="btn btn-primary">
              Add
            </button>
          </div>
        </form>

        {accounts.length === 0 ? (
          <p className="module-empty">Add an account to start the ledger.</p>
        ) : (
          <ul className="module-list">
            {accounts.map((acct) => (
              <li key={acct.id}>
                <div className="module-row-main">
                  <p className="module-row-title">{acct.name}</p>
                  <p className="module-row-meta">
                    {acct.kind} · {formatMoney(acct.balance, acct.currency)}
                  </p>
                </div>
                <div className="row-actions">
                  <button
                    type="button"
                    className="btn btn-danger btn-icon"
                    onClick={() => void onDeleteAccount(acct.id)}
                    aria-label={`Delete ${acct.name}`}
                  >
                    Del
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}

        <form onSubmit={onCreateTxn}>
          <p className="module-eyebrow finance-subhead">Add transaction</p>
          <div className="field-row">
            <select
              className="field-select"
              value={txnAccountId === "" ? "" : String(txnAccountId)}
              onChange={(e) =>
                setTxnAccountId(e.target.value ? Number(e.target.value) : "")
              }
              aria-label="Transaction account"
            >
              <option value="">Account</option>
              {accounts.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
            <input
              className="field"
              value={txnAmount}
              onChange={(e) => setTxnAmount(e.target.value)}
              placeholder="Amount (− spend)"
              aria-label="Amount"
              inputMode="decimal"
            />
            <input
              className="field"
              type="date"
              value={txnDate}
              onChange={(e) => setTxnDate(e.target.value)}
              aria-label="Posted date"
            />
          </div>
          <div className="field-row">
            <input
              className="field"
              value={txnDesc}
              onChange={(e) => setTxnDesc(e.target.value)}
              placeholder="Description"
              aria-label="Description"
            />
            <button type="submit" className="btn btn-primary">
              Add
            </button>
          </div>
        </form>

        <div>
          <p className="module-eyebrow finance-subhead">Import CSV</p>
          <div className="field-row">
            <select
              className="field-select"
              value={importAccountId === "" ? "" : String(importAccountId)}
              onChange={(e) =>
                setImportAccountId(
                  e.target.value ? Number(e.target.value) : "",
                )
              }
              aria-label="Import account"
            >
              <option value="">Account</option>
              {accounts.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
            <select
              className="field-select"
              value={importFormat}
              onChange={(e) => setImportFormat(e.target.value as CsvFormat)}
              aria-label="CSV format"
            >
              <option value="auto">Auto-detect</option>
              <option value="apple_card">Apple Card</option>
              <option value="chase">Chase</option>
              <option value="bofa">Bank of America</option>
              <option value="generic">Generic</option>
            </select>
            <input
              ref={fileRef}
              className="field"
              type="file"
              accept=".csv,text/csv"
              aria-label="CSV file"
              onChange={(e) =>
                void onImportFile(e.target.files?.[0] ?? null)
              }
            />
          </div>
        </div>

        <p className="module-eyebrow finance-subhead">Transactions</p>
        {transactions.length === 0 ? (
          <p className="module-empty">No transactions yet.</p>
        ) : (
          <ul className="module-list">
            {transactions.map((txn) => (
              <li key={txn.id}>
                <div className="module-row-main">
                  <p className="module-row-title">
                    {txn.description || "Transaction"}
                  </p>
                  <p className="module-row-meta">
                    {txn.accountName} · {formatPosted(txn.postedAt)}
                    {txn.categoryName ? ` · ${txn.categoryName}` : ""}
                  </p>
                </div>
                <p
                  className={`finance-amount ${txn.amount < 0 ? "is-neg" : ""}`}
                >
                  {formatMoney(txn.amount)}
                </p>
                <div className="row-actions">
                  <button
                    type="button"
                    className="btn btn-danger btn-icon"
                    onClick={() => void onDeleteTxn(txn.id)}
                    aria-label="Delete transaction"
                  >
                    Del
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}

        {bankLinks()}
      </DetailDrawer>
    </>
  );
}
