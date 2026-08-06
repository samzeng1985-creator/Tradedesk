use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::domain::{
    BusinessCase, BusinessCaseInput, BusinessCaseLine, Customer, CustomerInput, MilestoneStatus,
    PipelineStage, Product, ProductInput, ProductionMilestone, ProductionMilestoneInput,
    PurchaseOrder, PurchaseOrderInput, PurchaseOrderLine, PurchaseStatus, Supplier, SupplierInput,
    WorkspaceSummary,
};

const SCHEMA_VERSION: i64 = 4;

const MILESTONE_STAGES: [(&str, &str); 6] = [
    ("raw_material", "原料准备"),
    ("started", "开工"),
    ("production", "生产完成"),
    ("quality", "质检"),
    ("packing", "包装"),
    ("ready_to_ship", "可发货"),
];

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
                ON trade_case_lines(trade_case_id, sort_order);

             CREATE TABLE IF NOT EXISTS purchase_orders (
                id TEXT PRIMARY KEY,
                number TEXT NOT NULL UNIQUE,
                trade_case_id TEXT NOT NULL REFERENCES trade_cases(id),
                trade_case_number_snapshot TEXT NOT NULL,
                supplier_id TEXT NOT NULL REFERENCES suppliers(id),
                supplier_name_snapshot TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                currency TEXT NOT NULL,
                expected_date TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                total_amount_minor INTEGER NOT NULL DEFAULT 0,
                active INTEGER NOT NULL DEFAULT 1
             );
             CREATE INDEX IF NOT EXISTS idx_purchase_orders_case
                ON purchase_orders(trade_case_id, active);

             CREATE TABLE IF NOT EXISTS purchase_order_lines (
                id TEXT PRIMARY KEY,
                purchase_order_id TEXT NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
                source_case_line_id TEXT NOT NULL REFERENCES trade_case_lines(id),
                product_id TEXT NOT NULL,
                sku_snapshot TEXT NOT NULL,
                name_zh_snapshot TEXT NOT NULL,
                name_en_snapshot TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit_snapshot TEXT NOT NULL,
                unit_cost_minor INTEGER NOT NULL,
                amount_minor INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_purchase_order_lines_order
                ON purchase_order_lines(purchase_order_id);
             CREATE INDEX IF NOT EXISTS idx_purchase_order_lines_source
                ON purchase_order_lines(source_case_line_id);

             CREATE TABLE IF NOT EXISTS production_milestones (
                id TEXT PRIMARY KEY,
                purchase_order_line_id TEXT NOT NULL REFERENCES purchase_order_lines(id) ON DELETE CASCADE,
                stage TEXT NOT NULL,
                sort_order INTEGER NOT NULL,
                label TEXT NOT NULL,
                planned_date TEXT NOT NULL DEFAULT '',
                actual_date TEXT NOT NULL DEFAULT '',
                progress INTEGER NOT NULL DEFAULT 0,
                completed_quantity REAL NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                issue TEXT NOT NULL DEFAULT '',
                UNIQUE(purchase_order_line_id, stage)
             );
             CREATE INDEX IF NOT EXISTS idx_production_milestones_line
                ON production_milestones(purchase_order_line_id, sort_order);",
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
            purchase_orders: self.count("purchase_orders")?,
            production_risks: self.count("production_risks")?,
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
            if let Some(existing_line_id) = &line.id {
                let existing_product_id = transaction
                    .query_row(
                        "SELECT product_id FROM trade_case_lines
                         WHERE id = ?1 AND trade_case_id = ?2",
                        params![existing_line_id, id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()?;
                if let Some(existing_product_id) = existing_product_id {
                    let allocated = transaction.query_row(
                        "SELECT COALESCE(SUM(pol.quantity), 0)
                         FROM purchase_order_lines pol
                         JOIN purchase_orders po ON po.id = pol.purchase_order_id
                         WHERE pol.source_case_line_id = ?1
                           AND po.active = 1 AND po.status <> 'cancelled'",
                        params![existing_line_id],
                        |row| row.get::<_, f64>(0),
                    )?;
                    if allocated > 0.0
                        && (existing_product_id != line.product_id
                            || line.quantity + 0.000_001 < allocated)
                    {
                        return Err(rusqlite::Error::InvalidQuery);
                    }
                }
            }
            let amount_minor = (line.quantity * line.unit_price_minor as f64).round() as i64;
            total_amount_minor = total_amount_minor
                .checked_add(amount_minor)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            let line_id = line
                .id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            prepared_lines.push((line, line_id, product, amount_minor));
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
        let existing_line_ids = {
            let mut statement =
                transaction.prepare("SELECT id FROM trade_case_lines WHERE trade_case_id = ?1")?;
            statement
                .query_map(params![id], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for existing_id in existing_line_ids {
            if !prepared_lines
                .iter()
                .any(|(_, line_id, _, _)| line_id == &existing_id)
            {
                transaction.execute(
                    "DELETE FROM trade_case_lines WHERE id = ?1 AND trade_case_id = ?2",
                    params![existing_id, id],
                )?;
            }
        }
        for (index, (line, line_id, product, amount_minor)) in prepared_lines.iter().enumerate() {
            transaction.execute(
                "INSERT INTO trade_case_lines(
                    id, trade_case_id, sort_order, product_id, sku_snapshot,
                    name_zh_snapshot, name_en_snapshot, quantity, unit_snapshot,
                    unit_price_minor, amount_minor
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(id) DO UPDATE SET
                    sort_order = excluded.sort_order,
                    product_id = excluded.product_id,
                    sku_snapshot = excluded.sku_snapshot,
                    name_zh_snapshot = excluded.name_zh_snapshot,
                    name_en_snapshot = excluded.name_en_snapshot,
                    quantity = excluded.quantity,
                    unit_snapshot = excluded.unit_snapshot,
                    unit_price_minor = excluded.unit_price_minor,
                    amount_minor = excluded.amount_minor",
                params![
                    line_id,
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

    pub fn list_purchase_orders(&self) -> rusqlite::Result<Vec<PurchaseOrder>> {
        let ids = {
            let mut statement = self.connection.prepare(
                "SELECT id FROM purchase_orders WHERE active = 1
                 ORDER BY number COLLATE NOCASE DESC",
            )?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        ids.iter().map(|id| self.get_purchase_order(id)).collect()
    }

    pub fn get_purchase_order(&self, id: &str) -> rusqlite::Result<PurchaseOrder> {
        let mut purchase_order = self.connection.query_row(
            "SELECT id, number, trade_case_id, trade_case_number_snapshot, supplier_id,
                    supplier_name_snapshot, status, currency, expected_date, notes,
                    total_amount_minor
             FROM purchase_orders WHERE id = ?1 AND active = 1",
            params![id],
            |row| {
                let status_value: String = row.get(6)?;
                let status =
                    PurchaseStatus::from_db(&status_value).ok_or(rusqlite::Error::InvalidQuery)?;
                Ok(PurchaseOrder {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    business_case_id: row.get(2)?,
                    business_case_number: row.get(3)?,
                    supplier_id: row.get(4)?,
                    supplier_name: row.get(5)?,
                    status,
                    currency: row.get(7)?,
                    expected_date: row.get(8)?,
                    notes: row.get(9)?,
                    total_amount_minor: row.get(10)?,
                    completed_quantity: 0.0,
                    ready_quantity: 0.0,
                    lines: Vec::new(),
                })
            },
        )?;

        let line_ids = {
            let mut statement = self.connection.prepare(
                "SELECT id FROM purchase_order_lines WHERE purchase_order_id = ?1 ORDER BY rowid",
            )?;
            statement
                .query_map(params![id], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for line_id in line_ids {
            let mut line = self.connection.query_row(
                "SELECT id, source_case_line_id, product_id, sku_snapshot, name_zh_snapshot,
                        name_en_snapshot, quantity, unit_snapshot, unit_cost_minor, amount_minor
                 FROM purchase_order_lines WHERE id = ?1",
                params![line_id],
                |row| {
                    Ok(PurchaseOrderLine {
                        id: row.get(0)?,
                        source_case_line_id: row.get(1)?,
                        product_id: row.get(2)?,
                        sku: row.get(3)?,
                        name_zh: row.get(4)?,
                        name_en: row.get(5)?,
                        quantity: row.get(6)?,
                        unit: row.get(7)?,
                        unit_cost_minor: row.get(8)?,
                        amount_minor: row.get(9)?,
                        milestones: Vec::new(),
                    })
                },
            )?;
            let mut milestone_statement = self.connection.prepare(
                "SELECT id, purchase_order_line_id, stage, label, planned_date, actual_date,
                        progress, completed_quantity, status, issue
                 FROM production_milestones
                 WHERE purchase_order_line_id = ?1 ORDER BY sort_order",
            )?;
            line.milestones = milestone_statement
                .query_map(params![line.id], map_milestone)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            purchase_order.completed_quantity += line
                .milestones
                .iter()
                .find(|milestone| milestone.stage == "production")
                .map(|milestone| milestone.completed_quantity)
                .unwrap_or(0.0);
            purchase_order.ready_quantity += line
                .milestones
                .iter()
                .find(|milestone| milestone.stage == "ready_to_ship")
                .map(|milestone| milestone.completed_quantity)
                .unwrap_or(0.0);
            purchase_order.lines.push(line);
        }
        Ok(purchase_order)
    }

    pub fn create_purchase_order(
        &self,
        input: PurchaseOrderInput,
    ) -> rusqlite::Result<PurchaseOrder> {
        require_text(&input.number)?;
        require_text(&input.business_case_id)?;
        require_text(&input.supplier_id)?;
        require_text(&input.currency)?;
        if input.lines.is_empty()
            || input.lines.iter().any(|line| {
                line.source_case_line_id.trim().is_empty()
                    || !line.quantity.is_finite()
                    || line.quantity <= 0.0
                    || line.unit_cost_minor < 0
            })
        {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let transaction = self.connection.unchecked_transaction()?;
        let case_number = transaction.query_row(
            "SELECT number FROM trade_cases WHERE id = ?1 AND active = 1",
            params![input.business_case_id],
            |row| row.get::<_, String>(0),
        )?;
        let supplier_name = transaction.query_row(
            "SELECT legal_name FROM suppliers WHERE id = ?1 AND active = 1",
            params![input.supplier_id],
            |row| row.get::<_, String>(0),
        )?;
        let mut prepared_lines = Vec::with_capacity(input.lines.len());
        let mut total_amount_minor = 0_i64;
        for line in &input.lines {
            let snapshot = transaction.query_row(
                "SELECT product_id, sku_snapshot, name_zh_snapshot, name_en_snapshot,
                        quantity, unit_snapshot
                 FROM trade_case_lines WHERE id = ?1 AND trade_case_id = ?2",
                params![line.source_case_line_id, input.business_case_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )?;
            let allocated = transaction.query_row(
                "SELECT COALESCE(SUM(pol.quantity), 0)
                 FROM purchase_order_lines pol
                 JOIN purchase_orders po ON po.id = pol.purchase_order_id
                 WHERE pol.source_case_line_id = ?1 AND po.active = 1 AND po.status <> 'cancelled'",
                params![line.source_case_line_id],
                |row| row.get::<_, f64>(0),
            )?;
            if allocated + line.quantity > snapshot.4 + 0.000_001 {
                return Err(rusqlite::Error::InvalidQuery);
            }
            let amount_minor = (line.quantity * line.unit_cost_minor as f64).round() as i64;
            total_amount_minor = total_amount_minor
                .checked_add(amount_minor)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            prepared_lines.push((line, snapshot, amount_minor));
        }

        let id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO purchase_orders(
                id, number, trade_case_id, trade_case_number_snapshot, supplier_id,
                supplier_name_snapshot, status, currency, expected_date, notes,
                total_amount_minor, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'draft', ?7, ?8, ?9, ?10, 1)",
            params![
                id,
                input.number.trim(),
                input.business_case_id,
                case_number,
                input.supplier_id,
                supplier_name,
                input.currency.trim().to_uppercase(),
                input.expected_date.trim(),
                input.notes.trim(),
                total_amount_minor,
            ],
        )?;
        for (line, snapshot, amount_minor) in prepared_lines {
            let line_id = Uuid::new_v4().to_string();
            transaction.execute(
                "INSERT INTO purchase_order_lines(
                    id, purchase_order_id, source_case_line_id, product_id, sku_snapshot,
                    name_zh_snapshot, name_en_snapshot, quantity, unit_snapshot,
                    unit_cost_minor, amount_minor
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    line_id,
                    id,
                    line.source_case_line_id,
                    snapshot.0,
                    snapshot.1,
                    snapshot.2,
                    snapshot.3,
                    line.quantity,
                    snapshot.5,
                    line.unit_cost_minor,
                    amount_minor,
                ],
            )?;
            for (sort_order, (stage, label)) in MILESTONE_STAGES.iter().enumerate() {
                transaction.execute(
                    "INSERT INTO production_milestones(
                        id, purchase_order_line_id, stage, sort_order, label,
                        planned_date, actual_date, progress, completed_quantity, status, issue
                     ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, '', 0, 0, 'pending', '')",
                    params![
                        Uuid::new_v4().to_string(),
                        line_id,
                        stage,
                        sort_order as i64,
                        label,
                        input.expected_date.trim(),
                    ],
                )?;
            }
        }
        transaction.execute(
            "UPDATE trade_cases SET stage = 'purchase'
             WHERE id = ?1 AND stage IN ('quotation', 'order')",
            params![input.business_case_id],
        )?;
        update_case_purchase_amount(&transaction, &input.business_case_id)?;
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('purchase_order', ?1, 'create', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.get_purchase_order(&id)
    }

    pub fn update_purchase_order_status(
        &self,
        id: &str,
        status: PurchaseStatus,
    ) -> rusqlite::Result<PurchaseOrder> {
        let transaction = self.connection.unchecked_transaction()?;
        let case_id = transaction.query_row(
            "SELECT trade_case_id FROM purchase_orders WHERE id = ?1 AND active = 1",
            params![id],
            |row| row.get::<_, String>(0),
        )?;
        if status != PurchaseStatus::Cancelled {
            let lines = {
                let mut statement = transaction.prepare(
                    "SELECT source_case_line_id, quantity
                     FROM purchase_order_lines WHERE purchase_order_id = ?1",
                )?;
                statement
                    .query_map(params![id], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?
            };
            for (source_line_id, quantity) in lines {
                let sales_quantity = transaction.query_row(
                    "SELECT quantity FROM trade_case_lines WHERE id = ?1",
                    params![source_line_id],
                    |row| row.get::<_, f64>(0),
                )?;
                let allocated_elsewhere = transaction.query_row(
                    "SELECT COALESCE(SUM(pol.quantity), 0)
                     FROM purchase_order_lines pol
                     JOIN purchase_orders po ON po.id = pol.purchase_order_id
                     WHERE pol.source_case_line_id = ?1 AND po.id <> ?2
                       AND po.active = 1 AND po.status <> 'cancelled'",
                    params![source_line_id, id],
                    |row| row.get::<_, f64>(0),
                )?;
                if allocated_elsewhere + quantity > sales_quantity + 0.000_001 {
                    return Err(rusqlite::Error::InvalidQuery);
                }
            }
        }
        transaction.execute(
            "UPDATE purchase_orders SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        update_case_purchase_amount(&transaction, &case_id)?;
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('purchase_order', ?1, 'status', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.get_purchase_order(id)
    }

    pub fn update_production_milestone(
        &self,
        input: ProductionMilestoneInput,
    ) -> rusqlite::Result<ProductionMilestone> {
        if !(0..=100).contains(&input.progress)
            || !input.completed_quantity.is_finite()
            || input.completed_quantity < 0.0
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let transaction = self.connection.unchecked_transaction()?;
        let (line_quantity, purchase_order_id, case_id, stage) = transaction.query_row(
            "SELECT pol.quantity, po.id, po.trade_case_id, pm.stage
             FROM production_milestones pm
             JOIN purchase_order_lines pol ON pol.id = pm.purchase_order_line_id
             JOIN purchase_orders po ON po.id = pol.purchase_order_id
             WHERE pm.id = ?1 AND po.active = 1",
            params![input.id],
            |row| {
                Ok((
                    row.get::<_, f64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )?;
        if input.completed_quantity > line_quantity + 0.000_001 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let progress = if input.status == MilestoneStatus::Completed {
            100
        } else {
            input.progress
        };
        transaction.execute(
            "UPDATE production_milestones SET
                planned_date = ?1, actual_date = ?2, progress = ?3,
                completed_quantity = ?4, status = ?5, issue = ?6
             WHERE id = ?7",
            params![
                input.planned_date.trim(),
                input.actual_date.trim(),
                progress,
                input.completed_quantity,
                input.status.as_str(),
                input.issue.trim(),
                input.id,
            ],
        )?;
        if stage == "ready_to_ship" && input.status == MilestoneStatus::Completed {
            let remaining = transaction.query_row(
                "SELECT COUNT(*) FROM production_milestones pm
                 JOIN purchase_order_lines pol ON pol.id = pm.purchase_order_line_id
                 WHERE pol.purchase_order_id = ?1 AND pm.stage = 'ready_to_ship'
                   AND pm.status <> 'completed'",
                params![purchase_order_id],
                |row| row.get::<_, i64>(0),
            )?;
            if remaining == 0 {
                transaction.execute(
                    "UPDATE purchase_orders SET status = 'ready_to_ship'
                     WHERE id = ?1 AND status NOT IN ('completed', 'cancelled')",
                    params![purchase_order_id],
                )?;
            }
        } else if progress > 0 {
            transaction.execute(
                "UPDATE purchase_orders SET status = 'in_production'
                 WHERE id = ?1 AND status NOT IN ('ready_to_ship', 'completed', 'cancelled')",
                params![purchase_order_id],
            )?;
        }
        transaction.execute(
            "UPDATE trade_cases SET stage = 'production'
             WHERE id = ?1 AND stage IN ('quotation', 'order', 'purchase')",
            params![case_id],
        )?;
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('production_milestone', ?1, 'update', '{}')",
            params![input.id],
        )?;
        transaction.commit()?;
        self.connection.query_row(
            "SELECT id, purchase_order_line_id, stage, label, planned_date, actual_date,
                    progress, completed_quantity, status, issue
             FROM production_milestones WHERE id = ?1",
            params![input.id],
            map_milestone,
        )
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
            "purchase_orders" => {
                "SELECT COUNT(*) FROM purchase_orders WHERE active = 1 AND status <> 'cancelled'"
            }
            "production_risks" => {
                "SELECT COUNT(*) FROM production_milestones pm
                JOIN purchase_order_lines pol ON pol.id = pm.purchase_order_line_id
                JOIN purchase_orders po ON po.id = pol.purchase_order_id
                WHERE po.active = 1 AND po.status <> 'cancelled' AND pm.status = 'blocked'"
            }
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let count = self
            .connection
            .query_row(query, [], |row| row.get::<_, i64>(0))?;
        Ok(count as u64)
    }
}

fn map_milestone(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProductionMilestone> {
    let status_value: String = row.get(8)?;
    let status = MilestoneStatus::from_db(&status_value).ok_or(rusqlite::Error::InvalidQuery)?;
    Ok(ProductionMilestone {
        id: row.get(0)?,
        purchase_order_line_id: row.get(1)?,
        stage: row.get(2)?,
        label: row.get(3)?,
        planned_date: row.get(4)?,
        actual_date: row.get(5)?,
        progress: row.get(6)?,
        completed_quantity: row.get(7)?,
        status,
        issue: row.get(9)?,
    })
}

fn update_case_purchase_amount(
    transaction: &Transaction<'_>,
    case_id: &str,
) -> rusqlite::Result<()> {
    transaction.execute(
        "UPDATE trade_cases SET purchase_amount_minor = (
            SELECT COALESCE(SUM(total_amount_minor), 0)
            FROM purchase_orders
            WHERE trade_case_id = ?1 AND active = 1 AND status <> 'cancelled'
         ) WHERE id = ?1",
        params![case_id],
    )?;
    Ok(())
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
    use crate::domain::BusinessCaseLineInput;

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
            let supplier = database
                .save_supplier(SupplierInput {
                    id: None,
                    code: "SUP-1".into(),
                    legal_name: "Example Factory".into(),
                    lead_time_days: 20,
                    on_time_rate: 95,
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
                        id: None,
                        product_id: product.id,
                        quantity: 12.5,
                        unit_price_minor: 240,
                    }],
                })
                .unwrap();
            assert_eq!(business_case.total_amount_minor, 3_000);
            let purchase_order = database
                .create_purchase_order(PurchaseOrderInput {
                    number: "PO-2026-0001".into(),
                    business_case_id: business_case.id.clone(),
                    supplier_id: supplier.id,
                    currency: "USD".into(),
                    expected_date: "2026-09-01".into(),
                    notes: "first purchase".into(),
                    lines: vec![crate::domain::PurchaseOrderLineInput {
                        source_case_line_id: business_case.lines[0].id.clone(),
                        quantity: 12.5,
                        unit_cost_minor: 160,
                    }],
                })
                .unwrap();
            assert_eq!(purchase_order.total_amount_minor, 2_000);
            assert_eq!(purchase_order.lines[0].milestones.len(), 6);
            assert!(
                database
                    .save_business_case(BusinessCaseInput {
                        id: Some(business_case.id.clone()),
                        number: business_case.number.clone(),
                        customer_id: business_case.customer_id.clone(),
                        stage: business_case.stage.clone(),
                        currency: business_case.currency.clone(),
                        incoterm: business_case.incoterm.clone(),
                        payment_terms: business_case.payment_terms.clone(),
                        shipment_date: business_case.shipment_date.clone(),
                        notes: business_case.notes.clone(),
                        lines: vec![BusinessCaseLineInput {
                            id: Some(business_case.lines[0].id.clone()),
                            product_id: business_case.lines[0].product_id.clone(),
                            quantity: 10.0,
                            unit_price_minor: business_case.lines[0].unit_price_minor,
                        }],
                    })
                    .is_err()
            );
            let ready = purchase_order.lines[0]
                .milestones
                .iter()
                .find(|milestone| milestone.stage == "ready_to_ship")
                .unwrap();
            database
                .update_production_milestone(ProductionMilestoneInput {
                    id: ready.id.clone(),
                    planned_date: "2026-09-01".into(),
                    actual_date: "2026-08-31".into(),
                    progress: 100,
                    completed_quantity: 12.5,
                    status: MilestoneStatus::Completed,
                    issue: String::new(),
                })
                .unwrap();
            assert!(
                database
                    .create_purchase_order(PurchaseOrderInput {
                        number: "PO-2026-0002".into(),
                        business_case_id: business_case.id.clone(),
                        supplier_id: purchase_order.supplier_id.clone(),
                        currency: "USD".into(),
                        expected_date: "2026-09-02".into(),
                        notes: String::new(),
                        lines: vec![crate::domain::PurchaseOrderLineInput {
                            source_case_line_id: business_case.lines[0].id.clone(),
                            quantity: 1.0,
                            unit_cost_minor: 160,
                        }],
                    })
                    .is_err()
            );
            assert_eq!(database.summary().unwrap().products, 1);
            assert_eq!(database.summary().unwrap().active_cases, 1);
            assert_eq!(database.summary().unwrap().purchase_orders, 1);
        }
        let reopened =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        assert_eq!(reopened.list_products().unwrap()[0].sku, "SKU-1");
        let business_cases = reopened.list_business_cases().unwrap();
        assert_eq!(business_cases[0].number, "TD-2026-0001");
        assert_eq!(business_cases[0].lines[0].name_en, "Test product");
        let purchase_orders = reopened.list_purchase_orders().unwrap();
        assert_eq!(purchase_orders[0].number, "PO-2026-0001");
        assert_eq!(purchase_orders[0].ready_quantity, 12.5);
        drop(reopened);

        let header = std::fs::read(&path).unwrap();
        assert_ne!(&header[..16], b"SQLite format 3\0");
        assert!(
            EncryptedDatabase::open(&path, Zeroizing::new("wrong-password".to_owned())).is_err()
        );
        let _ = std::fs::remove_file(&path);
    }
}
