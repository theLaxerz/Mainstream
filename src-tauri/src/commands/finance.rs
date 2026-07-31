use crate::db::{now_iso, DbError, DbState};
use chrono::{NaiveDate, Utc};
use csv::ReaderBuilder;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub currency: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWithBalance {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub currency: String,
    pub created_at: String,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: f64,
    pub description: Option<String>,
    pub posted_at: String,
    pub created_at: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionView {
    pub id: i64,
    pub account_id: i64,
    pub account_name: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub amount: f64,
    pub description: Option<String>,
    pub posted_at: String,
    pub created_at: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceSummary {
    pub cash_total: f64,
    pub net_total: f64,
    pub accounts: Vec<AccountWithBalance>,
    pub recent: Vec<TransactionView>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountInput {
    pub name: String,
    pub kind: String,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountInput {
    pub id: i64,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionInput {
    pub account_id: i64,
    pub amount: f64,
    pub description: Option<String>,
    pub posted_at: Option<String>,
    pub category_id: Option<i64>,
    pub external_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransactionInput {
    pub id: i64,
    pub account_id: Option<i64>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub posted_at: Option<String>,
    pub category_id: Option<Option<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCsvInput {
    pub account_id: i64,
    pub csv_text: String,
    /// Optional hint: "apple_card" | "chase" | "bofa" | "generic" | auto-detect when omitted.
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCsvResult {
    pub imported: u32,
    pub skipped: u32,
    pub format: String,
}

fn validate_account_kind(kind: &str) -> Result<(), DbError> {
    match kind {
        "checking" | "savings" | "cash" | "credit" | "investment" | "other" => Ok(()),
        _ => Err(DbError::Message(
            "kind must be checking, savings, cash, credit, investment, or other".into(),
        )),
    }
}

fn map_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
        currency: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn map_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Transaction> {
    Ok(Transaction {
        id: row.get(0)?,
        account_id: row.get(1)?,
        category_id: row.get(2)?,
        amount: row.get(3)?,
        description: row.get(4)?,
        posted_at: row.get(5)?,
        created_at: row.get(6)?,
        external_id: row.get(7)?,
    })
}

fn map_transaction_view(row: &rusqlite::Row<'_>) -> rusqlite::Result<TransactionView> {
    Ok(TransactionView {
        id: row.get(0)?,
        account_id: row.get(1)?,
        account_name: row.get(2)?,
        category_id: row.get(3)?,
        category_name: row.get(4)?,
        amount: row.get(5)?,
        description: row.get(6)?,
        posted_at: row.get(7)?,
        created_at: row.get(8)?,
        external_id: row.get(9)?,
    })
}

fn get_account(conn: &Connection, id: i64) -> Result<Option<Account>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, kind, currency, created_at FROM accounts WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_account(row)?))
    } else {
        Ok(None)
    }
}

fn get_transaction(conn: &Connection, id: i64) -> Result<Option<Transaction>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, category_id, amount, description, posted_at, created_at, external_id
         FROM transactions WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_transaction(row)?))
    } else {
        Ok(None)
    }
}

fn list_accounts_with_balances(conn: &Connection) -> Result<Vec<AccountWithBalance>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.kind, a.currency, a.created_at,
                COALESCE(SUM(t.amount), 0) AS balance
         FROM accounts a
         LEFT JOIN transactions t ON t.account_id = a.id
         GROUP BY a.id
         ORDER BY a.name COLLATE NOCASE ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AccountWithBalance {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                currency: row.get(3)?,
                created_at: row.get(4)?,
                // Amounts are net-worth signed (purchases negative, income/payments positive).
                balance: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn list_recent_transactions(
    conn: &Connection,
    limit: i64,
    account_id: Option<i64>,
) -> Result<Vec<TransactionView>, DbError> {
    let sql = if account_id.is_some() {
        "SELECT t.id, t.account_id, a.name, t.category_id, c.name, t.amount, t.description,
                t.posted_at, t.created_at, t.external_id
         FROM transactions t
         JOIN accounts a ON a.id = t.account_id
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE t.account_id = ?1
         ORDER BY t.posted_at DESC, t.id DESC
         LIMIT ?2"
    } else {
        "SELECT t.id, t.account_id, a.name, t.category_id, c.name, t.amount, t.description,
                t.posted_at, t.created_at, t.external_id
         FROM transactions t
         JOIN accounts a ON a.id = t.account_id
         LEFT JOIN categories c ON c.id = t.category_id
         ORDER BY t.posted_at DESC, t.id DESC
         LIMIT ?1"
    };

    let mut stmt = conn.prepare(sql)?;
    let mapped = if let Some(aid) = account_id {
        stmt.query_map(params![aid, limit], map_transaction_view)?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(params![limit], map_transaction_view)?
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(mapped)
}

fn ensure_category(conn: &Connection, name: &str) -> Result<Option<i64>, DbError> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM categories WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
    {
        return Ok(Some(id));
    }
    conn.execute(
        "INSERT INTO categories (name, color) VALUES (?1, NULL)",
        params![name],
    )?;
    Ok(Some(conn.last_insert_rowid()))
}

fn external_exists(conn: &Connection, account_id: i64, external_id: &str) -> Result<bool, DbError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT id FROM transactions WHERE account_id = ?1 AND external_id = ?2",
            params![account_id, external_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

fn make_external_id(parts: &[&str]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for p in parts {
        p.hash(&mut hasher);
    }
    format!("h{:x}", hasher.finish())
}

fn parse_amount(raw: &str) -> Result<f64, DbError> {
    let cleaned = raw
        .trim()
        .trim_matches('"')
        .replace('$', "")
        .replace(',', "")
        .replace('(', "-")
        .replace(')', "");
    if cleaned.is_empty() {
        return Err(DbError::Message("empty amount".into()));
    }
    cleaned
        .parse::<f64>()
        .map_err(|e| DbError::Message(format!("invalid amount '{raw}': {e}")))
}

fn parse_date_to_iso(raw: &str) -> Result<String, DbError> {
    let s = raw.trim().trim_matches('"');
    if s.is_empty() {
        return Err(DbError::Message("empty date".into()));
    }
    // Already ISO-ish
    if s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') {
        let date = &s[..10];
        if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok() {
            return Ok(format!("{date}T12:00:00Z"));
        }
    }
    for fmt in ["%m/%d/%Y", "%m/%d/%y", "%Y-%m-%d", "%d/%m/%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Ok(format!("{}T12:00:00Z", d.format("%Y-%m-%d")));
        }
    }
    Err(DbError::Message(format!("unrecognized date '{raw}'")))
}

fn normalize_header(h: &str) -> String {
    h.trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .replace(['_', '-'], " ")
}

fn detect_csv_format(headers: &[String]) -> &'static str {
    let set: Vec<String> = headers.iter().map(|h| normalize_header(h)).collect();
    let has = |name: &str| set.iter().any(|h| h == name || h.contains(name));

    if has("amount (usd)") || (has("clearing date") && has("purchased by")) {
        "apple_card"
    } else if has("post date") && has("type") && has("amount") {
        "chase"
    } else if has("running bal") || (has("running balance") && has("date")) {
        "bofa"
    } else if has("amount") && (has("date") || has("transaction date") || has("posted date")) {
        "generic"
    } else {
        "generic"
    }
}

struct PendingTxn {
    amount: f64,
    description: String,
    posted_at: String,
    category: Option<String>,
    external_id: String,
}

fn header_index(headers: &[String]) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(i, h)| (normalize_header(h), i))
        .collect()
}

fn cell(record: &csv::StringRecord, idx: &HashMap<String, usize>, keys: &[&str]) -> String {
    for key in keys {
        if let Some(&i) = idx.get(*key) {
            if let Some(v) = record.get(i) {
                return v.trim().to_string();
            }
        }
    }
    // fuzzy contains
    for key in keys {
        for (h, &i) in idx {
            if h.contains(key) {
                if let Some(v) = record.get(i) {
                    return v.trim().to_string();
                }
            }
        }
    }
    String::new()
}

fn parse_csv_rows(csv_text: &str, format_hint: Option<&str>) -> Result<(String, Vec<PendingTxn>), DbError> {
    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_text.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| DbError::Message(format!("csv header error: {e}")))?
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>();

    if headers.is_empty() {
        return Err(DbError::Message("csv has no headers".into()));
    }

    let format = format_hint
        .filter(|f| !f.is_empty() && *f != "auto")
        .unwrap_or_else(|| detect_csv_format(&headers))
        .to_string();

    let idx = header_index(&headers);
    let mut pending = Vec::new();

    for (row_i, result) in reader.records().enumerate() {
        let record = result.map_err(|e| DbError::Message(format!("csv row {}: {e}", row_i + 2)))?;
        if record.iter().all(|c| c.trim().is_empty()) {
            continue;
        }

        let parsed = match format.as_str() {
            "apple_card" => parse_apple_card_row(&record, &idx),
            "chase" => parse_chase_row(&record, &idx),
            "bofa" => parse_bofa_row(&record, &idx),
            _ => parse_generic_row(&record, &idx),
        };

        match parsed {
            Ok(Some(txn)) => pending.push(txn),
            Ok(None) => {}
            Err(e) => {
                // Skip unparseable data rows rather than aborting the whole import.
                let _ = e;
            }
        }
    }

    Ok((format, pending))
}

fn parse_apple_card_row(
    record: &csv::StringRecord,
    idx: &HashMap<String, usize>,
) -> Result<Option<PendingTxn>, DbError> {
    let date = cell(record, idx, &["transaction date", "date"]);
    let desc = cell(record, idx, &["description", "merchant"]);
    let merchant = cell(record, idx, &["merchant"]);
    let category = cell(record, idx, &["category"]);
    let txn_type = cell(record, idx, &["type"]).to_ascii_lowercase();
    let amount_raw = cell(record, idx, &["amount (usd)", "amount"]);
    if date.is_empty() || amount_raw.is_empty() {
        return Ok(None);
    }
    let abs = parse_amount(&amount_raw)?.abs();
    // Apple Card exports positive amounts; type distinguishes direction.
    let amount = if txn_type.contains("payment") || txn_type.contains("credit") || txn_type.contains("refund")
    {
        abs
    } else {
        -abs
    };
    let description = if !merchant.is_empty() && merchant != desc {
        if desc.is_empty() {
            merchant
        } else {
            format!("{desc} — {merchant}")
        }
    } else if !desc.is_empty() {
        desc
    } else {
        merchant
    };
    let posted_at = parse_date_to_iso(&date)?;
    let external_id = make_external_id(&[
        "apple_card",
        &date,
        &description,
        &amount_raw,
        &txn_type,
    ]);
    Ok(Some(PendingTxn {
        amount,
        description,
        posted_at,
        category: if category.is_empty() {
            None
        } else {
            Some(category)
        },
        external_id,
    }))
}

fn parse_chase_row(
    record: &csv::StringRecord,
    idx: &HashMap<String, usize>,
) -> Result<Option<PendingTxn>, DbError> {
    let date = cell(record, idx, &["transaction date", "post date", "date"]);
    let desc = cell(record, idx, &["description"]);
    let category = cell(record, idx, &["category"]);
    let amount_raw = cell(record, idx, &["amount"]);
    if date.is_empty() || amount_raw.is_empty() {
        return Ok(None);
    }
    // Chase already signs amounts (purchases negative).
    let amount = parse_amount(&amount_raw)?;
    let posted_at = parse_date_to_iso(&date)?;
    let external_id = make_external_id(&["chase", &date, &desc, &amount_raw]);
    Ok(Some(PendingTxn {
        amount,
        description: desc,
        posted_at,
        category: if category.is_empty() {
            None
        } else {
            Some(category)
        },
        external_id,
    }))
}

fn parse_bofa_row(
    record: &csv::StringRecord,
    idx: &HashMap<String, usize>,
) -> Result<Option<PendingTxn>, DbError> {
    let date = cell(record, idx, &["date", "posted date", "transaction date"]);
    let desc = cell(record, idx, &["description", "payee"]);
    let amount_raw = cell(record, idx, &["amount"]);
    if date.is_empty() || amount_raw.is_empty() {
        return Ok(None);
    }
    let amount = parse_amount(&amount_raw)?;
    let posted_at = parse_date_to_iso(&date)?;
    let external_id = make_external_id(&["bofa", &date, &desc, &amount_raw]);
    Ok(Some(PendingTxn {
        amount,
        description: desc,
        posted_at,
        category: None,
        external_id,
    }))
}

fn parse_generic_row(
    record: &csv::StringRecord,
    idx: &HashMap<String, usize>,
) -> Result<Option<PendingTxn>, DbError> {
    let date = cell(
        record,
        idx,
        &[
            "transaction date",
            "posted date",
            "post date",
            "date",
            "posted",
        ],
    );
    let desc = cell(
        record,
        idx,
        &["description", "memo", "payee", "name", "merchant"],
    );
    let category = cell(record, idx, &["category"]);
    let amount_raw = cell(record, idx, &["amount", "amount (usd)", "transaction amount"]);
    let debit = cell(record, idx, &["debit", "withdrawal"]);
    let credit = cell(record, idx, &["credit", "deposit"]);

    if date.is_empty() {
        return Ok(None);
    }

    let amount = if !amount_raw.is_empty() {
        parse_amount(&amount_raw)?
    } else if !debit.is_empty() {
        -parse_amount(&debit)?.abs()
    } else if !credit.is_empty() {
        parse_amount(&credit)?.abs()
    } else {
        return Ok(None);
    };

    let posted_at = parse_date_to_iso(&date)?;
    let external_id = make_external_id(&["generic", &date, &desc, &amount.to_string()]);
    Ok(Some(PendingTxn {
        amount,
        description: desc,
        posted_at,
        category: if category.is_empty() {
            None
        } else {
            Some(category)
        },
        external_id,
    }))
}

// --- Commands ---

#[tauri::command]
pub fn list_accounts(state: State<'_, DbState>) -> Result<Vec<AccountWithBalance>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    list_accounts_with_balances(db.conn())
}

#[tauri::command]
pub fn create_account(
    state: State<'_, DbState>,
    input: CreateAccountInput,
) -> Result<Account, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err(DbError::Message("name is required".into()));
    }
    let kind = input.kind.trim().to_ascii_lowercase();
    validate_account_kind(&kind)?;
    let currency = input
        .currency
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .unwrap_or("USD");
    let now = now_iso();
    db.conn().execute(
        "INSERT INTO accounts (name, kind, currency, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, kind, currency, now],
    )?;
    let id = db.conn().last_insert_rowid();
    Ok(Account {
        id,
        name: name.to_string(),
        kind,
        currency: currency.to_string(),
        created_at: now,
    })
}

#[tauri::command]
pub fn update_account(
    state: State<'_, DbState>,
    input: UpdateAccountInput,
) -> Result<Account, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let existing = get_account(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("account {} not found", input.id)))?;

    let name = input
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or(existing.name);
    let kind = input
        .kind
        .map(|k| k.trim().to_ascii_lowercase())
        .unwrap_or(existing.kind);
    validate_account_kind(&kind)?;
    let currency = input
        .currency
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .unwrap_or(existing.currency);

    db.conn().execute(
        "UPDATE accounts SET name = ?1, kind = ?2, currency = ?3 WHERE id = ?4",
        params![name, kind, currency, input.id],
    )?;

    Ok(Account {
        id: input.id,
        name,
        kind,
        currency,
        created_at: existing.created_at,
    })
}

#[tauri::command]
pub fn delete_account(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::Message(format!("account {} not found", id)));
    }
    Ok(())
}

#[tauri::command]
pub fn list_categories(state: State<'_, DbState>) -> Result<Vec<Category>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db
        .conn()
        .prepare("SELECT id, name, color FROM categories ORDER BY name COLLATE NOCASE ASC")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn list_transactions(
    state: State<'_, DbState>,
    limit: Option<i64>,
    account_id: Option<i64>,
) -> Result<Vec<TransactionView>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    list_recent_transactions(db.conn(), limit.unwrap_or(100), account_id)
}

#[tauri::command]
pub fn create_transaction(
    state: State<'_, DbState>,
    input: CreateTransactionInput,
) -> Result<Transaction, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    if get_account(db.conn(), input.account_id)?.is_none() {
        return Err(DbError::Message(format!(
            "account {} not found",
            input.account_id
        )));
    }
    let now = now_iso();
    let posted_at = input
        .posted_at
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let description = input
        .description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());

    db.conn().execute(
        "INSERT INTO transactions
         (account_id, category_id, amount, description, posted_at, created_at, external_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            input.account_id,
            input.category_id,
            input.amount,
            description,
            posted_at,
            now,
            input.external_id
        ],
    )?;
    let id = db.conn().last_insert_rowid();
    Ok(Transaction {
        id,
        account_id: input.account_id,
        category_id: input.category_id,
        amount: input.amount,
        description,
        posted_at,
        created_at: now,
        external_id: input.external_id,
    })
}

#[tauri::command]
pub fn update_transaction(
    state: State<'_, DbState>,
    input: UpdateTransactionInput,
) -> Result<Transaction, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let existing = get_transaction(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("transaction {} not found", input.id)))?;

    let account_id = input.account_id.unwrap_or(existing.account_id);
    if get_account(db.conn(), account_id)?.is_none() {
        return Err(DbError::Message(format!("account {account_id} not found")));
    }
    let amount = input.amount.unwrap_or(existing.amount);
    let description = match input.description {
        Some(d) => {
            let t = d.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        }
        None => existing.description,
    };
    let posted_at = input
        .posted_at
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .unwrap_or(existing.posted_at);
    let category_id = match input.category_id {
        Some(v) => v,
        None => existing.category_id,
    };

    db.conn().execute(
        "UPDATE transactions
         SET account_id = ?1, category_id = ?2, amount = ?3, description = ?4, posted_at = ?5
         WHERE id = ?6",
        params![
            account_id,
            category_id,
            amount,
            description,
            posted_at,
            input.id
        ],
    )?;

    Ok(Transaction {
        id: input.id,
        account_id,
        category_id,
        amount,
        description,
        posted_at,
        created_at: existing.created_at,
        external_id: existing.external_id,
    })
}

#[tauri::command]
pub fn delete_transaction(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::Message(format!("transaction {} not found", id)));
    }
    Ok(())
}

#[tauri::command]
pub fn get_finance_summary(state: State<'_, DbState>) -> Result<FinanceSummary, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let accounts = list_accounts_with_balances(db.conn())?;
    let mut cash_total = 0.0;
    let mut net_total = 0.0;
    for acct in &accounts {
        net_total += acct.balance;
        // Cash snapshot: liquid accounts only (excludes credit, investment, other).
        if matches!(acct.kind.as_str(), "checking" | "savings" | "cash") {
            cash_total += acct.balance;
        }
    }
    let recent = list_recent_transactions(db.conn(), 10, None)?;
    Ok(FinanceSummary {
        cash_total,
        net_total,
        accounts,
        recent,
    })
}

#[tauri::command]
pub fn import_transactions_csv(
    state: State<'_, DbState>,
    input: ImportCsvInput,
) -> Result<ImportCsvResult, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    if get_account(db.conn(), input.account_id)?.is_none() {
        return Err(DbError::Message(format!(
            "account {} not found",
            input.account_id
        )));
    }

    let (format, pending) = parse_csv_rows(&input.csv_text, input.format.as_deref())?;
    let mut imported = 0u32;
    let mut skipped = 0u32;
    let now = now_iso();

    for txn in pending {
        if external_exists(db.conn(), input.account_id, &txn.external_id)? {
            skipped += 1;
            continue;
        }
        let category_id = match &txn.category {
            Some(name) => ensure_category(db.conn(), name)?,
            None => None,
        };
        let description = if txn.description.is_empty() {
            None
        } else {
            Some(txn.description)
        };
        db.conn().execute(
            "INSERT INTO transactions
             (account_id, category_id, amount, description, posted_at, created_at, external_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                input.account_id,
                category_id,
                txn.amount,
                description,
                txn.posted_at,
                now,
                txn.external_id
            ],
        )?;
        imported += 1;
    }

    Ok(ImportCsvResult {
        imported,
        skipped,
        format,
    })
}
