use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use zeroize::Zeroizing;

use crate::domain::WorkspaceSummary;

const SCHEMA_VERSION: i64 = 1;

pub struct EncryptedDatabase {
    connection: Connection,
}

impl EncryptedDatabase {
    pub fn open(path: &Path, key: Zeroizing<String>) -> rusqlite::Result<Self> {
        if key.is_empty() {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let connection = Connection::open(path)?;
        connection.pragma_update(None, "key", key.as_str())?;
        connection.execute_batch(
            "PRAGMA cipher_memory_security = ON;
             PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;",
        )?;

        let database = Self { connection };
        database.migrate()?;
        Ok(database)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS workspace_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS products (
                id TEXT PRIMARY KEY,
                sku TEXT NOT NULL UNIQUE,
                name_zh TEXT NOT NULL,
                name_en TEXT NOT NULL,
                hs_code TEXT NOT NULL DEFAULT '',
                unit TEXT NOT NULL,
                active INTEGER NOT NULL DEFAULT 1
             );

             CREATE TABLE IF NOT EXISTS customers (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                legal_name TEXT NOT NULL,
                market TEXT NOT NULL DEFAULT '',
                currency TEXT NOT NULL DEFAULT 'USD',
                active INTEGER NOT NULL DEFAULT 1
             );

             CREATE TABLE IF NOT EXISTS suppliers (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                legal_name TEXT NOT NULL,
                lead_time_days INTEGER NOT NULL DEFAULT 0,
                active INTEGER NOT NULL DEFAULT 1
             );

             CREATE TABLE IF NOT EXISTS trade_cases (
                id TEXT PRIMARY KEY,
                number TEXT NOT NULL UNIQUE,
                customer_id TEXT NOT NULL REFERENCES customers(id),
                stage TEXT NOT NULL,
                currency TEXT NOT NULL,
                sales_amount_minor INTEGER NOT NULL DEFAULT 0,
                purchase_amount_minor INTEGER NOT NULL DEFAULT 0
             );

             CREATE TABLE IF NOT EXISTS audit_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );",
        )?;
        transaction.execute(
            "INSERT INTO workspace_meta(key, value) VALUES('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SCHEMA_VERSION.to_string()],
        )?;
        transaction.commit()
    }

    pub fn summary(&self) -> rusqlite::Result<WorkspaceSummary> {
        let company_name = self
            .connection
            .query_row(
                "SELECT value FROM workspace_meta WHERE key = 'company_name'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(|| "未设置公司".to_owned());

        Ok(WorkspaceSummary {
            company_name,
            encrypted: true,
            products: self.count("products")?,
            customers: self.count("customers")?,
            suppliers: self.count("suppliers")?,
            active_cases: self.count("trade_cases")?,
        })
    }

    fn count(&self, table: &str) -> rusqlite::Result<u64> {
        let query = match table {
            "products" => "SELECT COUNT(*) FROM products WHERE active = 1",
            "customers" => "SELECT COUNT(*) FROM customers WHERE active = 1",
            "suppliers" => "SELECT COUNT(*) FROM suppliers WHERE active = 1",
            "trade_cases" => "SELECT COUNT(*) FROM trade_cases",
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let count = self
            .connection
            .query_row(query, [], |row| row.get::<_, i64>(0))?;
        Ok(count as u64)
    }
}
