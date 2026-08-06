use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::domain::{
    BusinessCase, BusinessCaseInput, BusinessCaseLine, BusinessCaseLineInput, Customer,
    CustomerInput, PipelineStage, Product, ProductInput, Supplier, SupplierInput, WorkspaceSummary,
};

const SCHEMA_VERSION: i64 = 3;

pub struct EncryptedDatabase {
    connection: Connection,
}

impl EncryptedDatabase {
    pub fn open(path: &Path, key: Zeroizing<String>) -> rusqlite::Result<Self> {
        if key.trim().len() < 8 {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let connection = Connection::open(path)?;
        connection.pragma_update(None, "key", key.as_str())?;
        #[cfg(not(target_os = "windows"))]
        connection.pragma_update(None, "cipher_memory_security", "ON")?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;",
        )?;
        connection.query_row("SELECT COUNT(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })?;

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

        ensure_column(
            &transaction,
            "products",
            "model",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "products",
            "gross_weight_kg",
            "REAL NOT NULL DEFAULT 0",
        )?;
        ensure_column(
            &transaction,
            "customers",
            "payment_terms",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "suppliers",
            "on_time_rate",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        ensure_column(
            &transaction,
            "trade_cases",
            "customer_name_snapshot",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "trade_cases",
            "incoterm",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "trade_cases",
            "payment_terms",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "trade_cases",
            "shipment_date",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "trade_cases",
            "notes",
            "TEXT NOT NULL DEFAULT ''",
        )?;
        ensure_column(
            &transaction,
            "trade_cases",
            "active",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS trade_case_lines (
                id TEXT PRIMARY KEY,
                trade_case_id TEXT NOT NULL REFERENCES trade_cases(id) ON DELETE CASCADE,
                sort_order INTEGER NOT NULL,
                product_id TEXT NOT NULL REFERENCES products(id),
                sku_snapshot TEXT NOT NULL,
                name_zh_snapshot TEXT NOT NULL,
                name_en_snapshot TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit_snapshot TEXT NOT NULL,
                unit_price_minor INTEGER NOT NULL,
                amount_minor INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_trade_case_lines_case
                ON trade_case_lines(trade_case_id, sort_order);",
        )?;

        transaction.execute(
            "INSERT INTO workspace_meta(key, value) VALUES('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SCHEMA_VERSION.to_string()],
        )?;
        transaction.commit()
    }

    pub fn initialize_company(&self, company_name: &str) -> rusqlite::Result<()> {
        if company_name.trim().is_empty() {
            return Ok(());
        }
        self.connection.execute(
            "INSERT INTO workspace_meta(key, value) VALUES('company_name', ?1)
             ON CONFLICT(key) DO NOTHING",
            params![company_name.trim()],
        )?;
        Ok(())
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
            .unwrap_or_else(|| "本地工作区".to_owned());

        Ok(WorkspaceSummary {
            company_name,
            encrypted: true,
            products: self.count("products")?,
            customers: self.count("customers")?,
            suppliers: self.count("suppliers")?,
            active_cases: self.count("trade_cases")?,
        })
    }

    pub fn list_products(&self) -> rusqlite::Result<Vec<Product>> {
        let mut statement = self.connection.prepare(
            "SELECT id, sku, name_zh, name_en, model, hs_code, unit, gross_weight_kg, active
             FROM products WHERE active = 1 ORDER BY sku COLLATE NOCASE",
        )?;
        statement
            .query_map([], |row| {
                Ok(Product {
                    id: row.get(0)?,
                    sku: row.get(1)?,
                    name_zh: row.get(2)?,
                    name_en: row.get(3)?,
                    model: row.get(4)?,
                    hs_code: row.get(5)?,
                    unit: row.get(6)?,
                    gross_weight_kg: row.get(7)?,
                    active: row.get::<_, i64>(8)? != 0,
                })
            })?
            .collect()
    }

    pub fn save_product(&self, input: ProductInput) -> rusqlite::Result<Product> {
        require_text(&input.sku)?;
        require_text(&input.name_en)?;
        require_text(&input.unit)?;
        if input.gross_weight_kg < 0.0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO products(id, sku, name_zh, name_en, model, hs_code, unit, gross_weight_kg, active)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)
             ON CONFLICT(id) DO UPDATE SET
                sku = excluded.sku, name_zh = excluded.name_zh, name_en = excluded.name_en,
                model = excluded.model, hs_code = excluded.hs_code, unit = excluded.unit,
                gross_weight_kg = excluded.gross_weight_kg, active = 1",
            params![id, input.sku.trim(), input.name_zh.trim(), input.name_en.trim(),
                input.model.trim(), input.hs_code.trim(), input.unit.trim(), input.gross_weight_kg],
        )?;
        self.audit("product", &id, "save")?;
        self.connection.query_row(
            "SELECT id, sku, name_zh, name_en, model, hs_code, unit, gross_weight_kg, active
             FROM products WHERE id = ?1",
            params![id],
            |row| {
                Ok(Product {
                    id: row.get(0)?,
                    sku: row.get(1)?,
                    name_zh: row.get(2)?,
                    name_en: row.get(3)?,
                    model: row.get(4)?,
                    hs_code: row.get(5)?,
                    unit: row.get(6)?,
                    gross_weight_kg: row.get(7)?,
                    active: row.get::<_, i64>(8)? != 0,
                })
            },
        )
    }

    pub fn list_customers(&self) -> rusqlite::Result<Vec<Customer>> {
        let mut statement = self.connection.prepare(
            "SELECT id, code, legal_name, market, currency, payment_terms, active
             FROM customers WHERE active = 1 ORDER BY code COLLATE NOCASE",
        )?;
        statement
            .query_map([], |row| {
                Ok(Customer {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    legal_name: row.get(2)?,
                    market: row.get(3)?,
                    currency: row.get(4)?,
                    payment_terms: row.get(5)?,
                    active: row.get::<_, i64>(6)? != 0,
                })
            })?
            .collect()
    }

    pub fn save_customer(&self, input: CustomerInput) -> rusqlite::Result<Customer> {
        require_text(&input.code)?;
        require_text(&input.legal_name)?;
        require_text(&input.currency)?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO customers(id, code, legal_name, market, currency, payment_terms, active)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 1)
             ON CONFLICT(id) DO UPDATE SET code = excluded.code, legal_name = excluded.legal_name,
                market = excluded.market, currency = excluded.currency,
                payment_terms = excluded.payment_terms, active = 1",
            params![
                id,
                input.code.trim(),
                input.legal_name.trim(),
                input.market.trim(),
                input.currency.trim().to_uppercase(),
                input.payment_terms.trim()
            ],
        )?;
        self.audit("customer", &id, "save")?;
        self.connection.query_row(
            "SELECT id, code, legal_name, market, currency, payment_terms, active FROM customers WHERE id = ?1",
            params![id],
            |row| Ok(Customer { id: row.get(0)?, code: row.get(1)?, legal_name: row.get(2)?,
                market: row.get(3)?, currency: row.get(4)?, payment_terms: row.get(5)?,
                active: row.get::<_, i64>(6)? != 0 }),
        )
    }

    pub fn list_suppliers(&self) -> rusqlite::Result<Vec<Supplier>> {
        let mut statement = self.connection.prepare(
            "SELECT id, code, legal_name, lead_time_days, on_time_rate, active
             FROM suppliers WHERE active = 1 ORDER BY code COLLATE NOCASE",
        )?;
        statement
            .query_map([], |row| {
                Ok(Supplier {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    legal_name: row.get(2)?,
                    lead_time_days: row.get(3)?,
                    on_time_rate: row.get(4)?,
                    active: row.get::<_, i64>(5)? != 0,
                })
            })?
            .collect()
    }

    pub fn save_supplier(&self, input: SupplierInput) -> rusqlite::Result<Supplier> {
        require_text(&input.code)?;
        require_text(&input.legal_name)?;
        if input.lead_time_days < 0 || !(0..=100).contains(&input.on_time_rate) {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO suppliers(id, code, legal_name, lead_time_days, on_time_rate, active)
             VALUES(?1, ?2, ?3, ?4, ?5, 1)
             ON CONFLICT(id) DO UPDATE SET code = excluded.code, legal_name = excluded.legal_name,
                lead_time_days = excluded.lead_time_days, on_time_rate = excluded.on_time_rate, active = 1",
            params![id, input.code.trim(), input.legal_name.trim(), input.lead_time_days, input.on_time_rate],
        )?;
        self.audit("supplier", &id, "save")?;
        self.connection.query_row(
            "SELECT id, code, legal_name, lead_time_days, on_time_rate, active FROM suppliers WHERE id = ?1",
            params![id],
            |row| Ok(Supplier { id: row.get(0)?, code: row.get(1)?, legal_name: row.get(2)?,
                lead_time_days: row.get(3)?, on_time_rate: row.get(4)?, active: row.get::<_, i64>(5)? != 0 }),
        )
    }

    pub fn archive(&self, entity: &str, id: &str) -> rusqlite::Result<()> {
        let table = match entity {
            "product" => "products",
            "customer" => "customers",
            "supplier" => "suppliers",
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let changed = self.connection.execute(
            &format!("UPDATE {table} SET active = 0 WHERE id = ?1"),
            params![id],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.audit(entity, id, "archive")
    }

    pub fn list_business_cases(&self) -> rusqlite::Result<Vec<BusinessCase>> {
        let ids = {
            let mut statement = self.connection.prepare(
                "SELECT id FROM trade_cases WHERE active = 1
                 ORDER BY number COLLATE NOCASE DESC",
            )?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        ids.iter().map(|id| self.get_business_case(id)).collect()
    }

    pub fn get_business_case(&self, id: &str) -> rusqlite::Result<BusinessCase> {
        let mut business_case = self.connection.query_row(
            "SELECT id, number, customer_id, customer_name_snapshot, stage, currency,
                    incoterm, payment_terms, shipment_date, notes, sales_amount_minor
             FROM trade_cases WHERE id = ?1 AND active = 1",
            params![id],
            |row| {
                let stage_value: String = row.get(4)?;
                let stage =
                    PipelineStage::from_db(&stage_value).ok_or(rusqlite::Error::InvalidQuery)?;
                Ok(BusinessCase {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    customer_id: row.get(2)?,
                    customer_name: row.get(3)?,
                    stage,
                    currency: row.get(5)?,
                    incoterm: row.get(6)?,
                    payment_terms: row.get(7)?,
                    shipment_date: row.get(8)?,
                    notes: row.get(9)?,
                    total_amount_minor: row.get(10)?,
                    lines: Vec::new(),
                })
            },
        )?;
        let mut statement = self.connection.prepare(
            "SELECT id, product_id, sku_snapshot, name_zh_snapshot, name_en_snapshot,
                    quantity, unit_snapshot, unit_price_minor, amount_minor
             FROM trade_case_lines WHERE trade_case_id = ?1 ORDER BY sort_order",
        )?;
        business_case.lines = statement
            .query_map(params![id], |row| {
                Ok(BusinessCaseLine {
                    id: row.get(0)?,
                    product_id: row.get(1)?,
                    sku: row.get(2)?,
                    name_zh: row.get(3)?,
                    name_en: row.get(4)?,
                    quantity: row.get(5)?,
                    unit: row.get(6)?,
                    unit_price_minor: row.get(7)?,
                    amount_minor: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(business_case)
    }

    pub fn save_business_case(&self, input: BusinessCaseInput) -> rusqlite::Result<BusinessCase> {
        require_text(&input.number)?;
        require_text(&input.customer_id)?;
        require_text(&input.currency)?;
        if input.lines.is_empty() {
            return Err(rusqlite::Error::InvalidQuery);
        }
        if input.lines.iter().any(|line| {
            line.product_id.trim().is_empty()
                || !line.quantity.is_finite()
                || line.quantity <= 0.0
                || line.unit_price_minor < 0
        }) {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let transaction = self.connection.unchecked_transaction()?;
        let customer_name = transaction.query_row(
            "SELECT legal_name FROM customers WHERE id = ?1 AND active = 1",
            params![input.customer_id],
            |row| row.get::<_, String>(0),
        )?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let mut prepared_lines = Vec::with_capacity(input.lines.len());
        let mut total_amount_minor = 0_i64;
        for line in &input.lines {
            let product = transaction.query_row(
                "SELECT sku, name_zh, name_en, unit FROM products WHERE id = ?1 AND active = 1",
                params![line.product_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )?;
            let amount_minor = (line.quantity * line.unit_price_minor as f64).round() as i64;
            total_amount_minor = total_amount_minor
                .checked_add(amount_minor)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            prepared_lines.push((line, product, amount_minor));
        }

        transaction.execute(
            "INSERT INTO trade_cases(
                id, number, customer_id, customer_name_snapshot, stage, currency,
                incoterm, payment_terms, shipment_date, notes,
                sales_amount_minor, purchase_amount_minor, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, 1)
             ON CONFLICT(id) DO UPDATE SET
                number = excluded.number, customer_id = excluded.customer_id,
                customer_name_snapshot = excluded.customer_name_snapshot,
                stage = excluded.stage, currency = excluded.currency,
                incoterm = excluded.incoterm, payment_terms = excluded.payment_terms,
                shipment_date = excluded.shipment_date, notes = excluded.notes,
                sales_amount_minor = excluded.sales_amount_minor, active = 1",
            params![
                id,
                input.number.trim(),
                input.customer_id,
                customer_name,
                input.stage.as_str(),
                input.currency.trim().to_uppercase(),
                input.incoterm.trim().to_uppercase(),
                input.payment_terms.trim(),
                input.shipment_date.trim(),
                input.notes.trim(),
                total_amount_minor,
            ],
        )?;
        transaction.execute(
            "DELETE FROM trade_case_lines WHERE trade_case_id = ?1",
            params![id],
        )?;
        for (index, (line, product, amount_minor)) in prepared_lines.iter().enumerate() {
            transaction.execute(
                "INSERT INTO trade_case_lines(
                    id, trade_case_id, sort_order, product_id, sku_snapshot,
                    name_zh_snapshot, name_en_snapshot, quantity, unit_snapshot,
                    unit_price_minor, amount_minor
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    index as i64,
                    line.product_id,
                    product.0,
                    product.1,
                    product.2,
                    line.quantity,
                    product.3,
                    line.unit_price_minor,
                    amount_minor,
                ],
            )?;
        }
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('business_case', ?1, 'save', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.get_business_case(&id)
    }

    pub fn archive_business_case(&self, id: &str) -> rusqlite::Result<()> {
        let changed = self.connection.execute(
            "UPDATE trade_cases SET active = 0 WHERE id = ?1",
            params![id],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.audit("business_case", id, "archive")
    }

    fn audit(&self, entity: &str, id: &str, action: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES(?1, ?2, ?3, '{}')",
            params![entity, id, action],
        )?;
        Ok(())
    }

    fn count(&self, table: &str) -> rusqlite::Result<u64> {
        let query = match table {
            "products" => "SELECT COUNT(*) FROM products WHERE active = 1",
            "customers" => "SELECT COUNT(*) FROM customers WHERE active = 1",
            "suppliers" => "SELECT COUNT(*) FROM suppliers WHERE active = 1",
            "trade_cases" => "SELECT COUNT(*) FROM trade_cases WHERE active = 1",
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let count = self
            .connection
            .query_row(query, [], |row| row.get::<_, i64>(0))?;
        Ok(count as u64)
    }
}

fn ensure_column(
    transaction: &Transaction<'_>,
    table: &str,
    column: &str,
    definition: &str,
) -> rusqlite::Result<()> {
    let mut statement = transaction.prepare(&format!("PRAGMA table_info({table})"))?;
    let exists = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .any(|name| name == column);
    drop(statement);
    if !exists {
        transaction.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {definition}"
        ))?;
    }
    Ok(())
}

fn require_text(value: &str) -> rusqlite::Result<()> {
    if value.trim().is_empty() {
        Err(rusqlite::Error::InvalidQuery)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_database_persists_business_workflow() {
        let path = std::env::temp_dir().join(format!("tradedesk-{}.db", Uuid::new_v4()));
        {
            let database =
                EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
            let product = database
                .save_product(ProductInput {
                    id: None,
                    sku: "SKU-1".into(),
                    name_zh: "测试产品".into(),
                    name_en: "Test product".into(),
                    model: "M1".into(),
                    hs_code: "0000.00".into(),
                    unit: "pcs".into(),
                    gross_weight_kg: 1.2,
                })
                .unwrap();
            let customer = database
                .save_customer(CustomerInput {
                    id: None,
                    code: "CUS-1".into(),
                    legal_name: "Example Import LLC".into(),
                    market: "US".into(),
                    currency: "USD".into(),
                    payment_terms: "30% deposit".into(),
                })
                .unwrap();
            let business_case = database
                .save_business_case(BusinessCaseInput {
                    id: None,
                    number: "TD-2026-0001".into(),
                    customer_id: customer.id,
                    stage: PipelineStage::Order,
                    currency: "USD".into(),
                    incoterm: "FOB".into(),
                    payment_terms: "30% deposit".into(),
                    shipment_date: "2026-09-18".into(),
                    notes: "test order".into(),
                    lines: vec![BusinessCaseLineInput {
                        product_id: product.id,
                        quantity: 12.5,
                        unit_price_minor: 240,
                    }],
                })
                .unwrap();
            assert_eq!(business_case.total_amount_minor, 3_000);
            assert_eq!(database.summary().unwrap().products, 1);
            assert_eq!(database.summary().unwrap().active_cases, 1);
        }
        let reopened =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        assert_eq!(reopened.list_products().unwrap()[0].sku, "SKU-1");
        let business_cases = reopened.list_business_cases().unwrap();
        assert_eq!(business_cases[0].number, "TD-2026-0001");
        assert_eq!(business_cases[0].lines[0].name_en, "Test product");
        drop(reopened);

        let header = std::fs::read(&path).unwrap();
        assert_ne!(&header[..16], b"SQLite format 3\0");
        assert!(
            EncryptedDatabase::open(&path, Zeroizing::new("wrong-password".to_owned())).is_err()
        );
        let _ = std::fs::remove_file(&path);
    }
}
