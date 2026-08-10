use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::domain::{
    BusinessCase, BusinessCaseInput, BusinessCaseLine, ConfigComponent, ConfigComponentInput,
    ConfigurableProduct, ConfigurableProductInput, ConfigurableProductLine, ConvertDocumentInput,
    CreateDocumentInput, Customer, CustomerInput, DocumentLineSnapshot, DocumentPayload,
    DocumentStatus, DocumentType, MilestoneStatus, PipelineStage, Product, ProductInput,
    ProductionMilestone, ProductionMilestoneInput, PurchaseOrder, PurchaseOrderInput,
    PurchaseOrderLine, PurchaseStatus, SaveDocumentInput, Supplier, SupplierInput, TradeDocument,
    WorkspaceSummary,
};

const SCHEMA_VERSION: i64 = 7;

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

             CREATE TABLE IF NOT EXISTS config_components (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                specification TEXT NOT NULL DEFAULT '',
                default_quantity REAL NOT NULL DEFAULT 1,
                unit TEXT NOT NULL,
                unit_price_minor INTEGER NOT NULL DEFAULT 0,
                currency TEXT NOT NULL DEFAULT 'CNY',
                brand TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1
             );
             CREATE INDEX IF NOT EXISTS idx_config_components_category
                ON config_components(category, code);

             CREATE TABLE IF NOT EXISTS configurable_products (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                model TEXT NOT NULL DEFAULT '',
                currency TEXT NOT NULL DEFAULT 'CNY',
                notes TEXT NOT NULL DEFAULT '',
                total_amount_minor INTEGER NOT NULL DEFAULT 0,
                active INTEGER NOT NULL DEFAULT 1
             );

             CREATE TABLE IF NOT EXISTS configurable_product_lines (
                id TEXT PRIMARY KEY,
                configurable_product_id TEXT NOT NULL REFERENCES configurable_products(id) ON DELETE CASCADE,
                sort_order INTEGER NOT NULL,
                component_id TEXT NOT NULL REFERENCES config_components(id),
                category_snapshot TEXT NOT NULL,
                name_snapshot TEXT NOT NULL,
                specification_snapshot TEXT NOT NULL DEFAULT '',
                quantity REAL NOT NULL,
                unit_snapshot TEXT NOT NULL,
                unit_price_minor INTEGER NOT NULL,
                brand_snapshot TEXT NOT NULL DEFAULT '',
                notes_snapshot TEXT NOT NULL DEFAULT '',
                amount_minor INTEGER NOT NULL,
                UNIQUE(configurable_product_id, component_id)
             );
             CREATE INDEX IF NOT EXISTS idx_configurable_product_lines_product
                ON configurable_product_lines(configurable_product_id, sort_order);

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
        for column in [
            "address",
            "shipping_address",
            "billing_address",
            "purchase_intent",
            "customer_analysis",
            "strengths",
            "weaknesses",
            "contacts",
        ] {
            ensure_column(
                &transaction,
                "customers",
                column,
                "TEXT NOT NULL DEFAULT ''",
            )?;
        }
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
                ON production_milestones(purchase_order_line_id, sort_order);

             CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                document_type TEXT NOT NULL,
                number TEXT NOT NULL,
                trade_case_id TEXT NOT NULL REFERENCES trade_cases(id),
                trade_case_number_snapshot TEXT NOT NULL,
                customer_name_snapshot TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'draft',
                language TEXT NOT NULL DEFAULT 'en',
                issue_date TEXT NOT NULL,
                currency TEXT NOT NULL,
                template_version TEXT NOT NULL DEFAULT 'base-1',
                payload_json TEXT NOT NULL,
                void_reason TEXT NOT NULL DEFAULT '',
                pdf_path TEXT NOT NULL DEFAULT '',
                pdf_sha256 TEXT NOT NULL DEFAULT '',
                exported_at TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(document_type, number, version)
             );
             CREATE INDEX IF NOT EXISTS idx_documents_case
                ON documents(trade_case_id, updated_at DESC);
             CREATE INDEX IF NOT EXISTS idx_documents_number
                ON documents(document_type, number, version DESC);",
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
            documents: self.count("documents")?,
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

    pub fn list_config_components(&self) -> rusqlite::Result<Vec<ConfigComponent>> {
        let mut statement = self.connection.prepare(
            "SELECT id, code, category, name, specification, default_quantity, unit,
                    unit_price_minor, currency, brand, notes, active
             FROM config_components WHERE active = 1
             ORDER BY category COLLATE NOCASE, code COLLATE NOCASE",
        )?;
        statement.query_map([], map_config_component)?.collect()
    }

    pub fn save_config_component(
        &self,
        input: ConfigComponentInput,
    ) -> rusqlite::Result<ConfigComponent> {
        require_text(&input.code)?;
        require_text(&input.category)?;
        require_text(&input.name)?;
        require_text(&input.unit)?;
        require_text(&input.currency)?;
        if input.default_quantity <= 0.0 || input.unit_price_minor < 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO config_components(
                id, code, category, name, specification, default_quantity, unit,
                unit_price_minor, currency, brand, notes, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)
             ON CONFLICT(id) DO UPDATE SET
                code = excluded.code, category = excluded.category, name = excluded.name,
                specification = excluded.specification, default_quantity = excluded.default_quantity,
                unit = excluded.unit, unit_price_minor = excluded.unit_price_minor,
                currency = excluded.currency, brand = excluded.brand, notes = excluded.notes,
                active = 1",
            params![
                id,
                input.code.trim(),
                input.category.trim(),
                input.name.trim(),
                input.specification.trim(),
                input.default_quantity,
                input.unit.trim(),
                input.unit_price_minor,
                input.currency.trim().to_uppercase(),
                input.brand.trim(),
                input.notes.trim(),
            ],
        )?;
        self.audit("config_component", &id, "save")?;
        self.connection.query_row(
            "SELECT id, code, category, name, specification, default_quantity, unit,
                    unit_price_minor, currency, brand, notes, active
             FROM config_components WHERE id = ?1",
            params![id],
            map_config_component,
        )
    }

    pub fn list_configurable_products(&self) -> rusqlite::Result<Vec<ConfigurableProduct>> {
        let ids = {
            let mut statement = self.connection.prepare(
                "SELECT id FROM configurable_products WHERE active = 1
                 ORDER BY code COLLATE NOCASE",
            )?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        ids.iter()
            .map(|id| self.get_configurable_product(id))
            .collect()
    }

    pub fn get_configurable_product(&self, id: &str) -> rusqlite::Result<ConfigurableProduct> {
        let mut product = self.connection.query_row(
            "SELECT id, code, name, model, currency, notes, total_amount_minor, active
             FROM configurable_products WHERE id = ?1",
            params![id],
            |row| {
                Ok(ConfigurableProduct {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    model: row.get(3)?,
                    currency: row.get(4)?,
                    notes: row.get(5)?,
                    total_amount_minor: row.get(6)?,
                    active: row.get::<_, i64>(7)? != 0,
                    lines: Vec::new(),
                })
            },
        )?;
        let mut statement = self.connection.prepare(
            "SELECT id, component_id, category_snapshot, name_snapshot,
                    specification_snapshot, quantity, unit_snapshot, unit_price_minor,
                    brand_snapshot, notes_snapshot, amount_minor
             FROM configurable_product_lines WHERE configurable_product_id = ?1
             ORDER BY sort_order",
        )?;
        product.lines = statement
            .query_map(params![id], map_configurable_product_line)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(product)
    }

    pub fn save_configurable_product(
        &self,
        input: ConfigurableProductInput,
    ) -> rusqlite::Result<ConfigurableProduct> {
        require_text(&input.code)?;
        require_text(&input.name)?;
        require_text(&input.currency)?;
        if input.lines.is_empty() {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let currency = input.currency.trim().to_uppercase();
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT INTO configurable_products(
                id, code, name, model, currency, notes, total_amount_minor, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 0, 1)
             ON CONFLICT(id) DO UPDATE SET code = excluded.code, name = excluded.name,
                model = excluded.model, currency = excluded.currency, notes = excluded.notes,
                active = 1",
            params![
                id,
                input.code.trim(),
                input.name.trim(),
                input.model.trim(),
                currency,
                input.notes.trim(),
            ],
        )?;
        transaction.execute(
            "DELETE FROM configurable_product_lines WHERE configurable_product_id = ?1",
            params![id],
        )?;

        let mut total_amount_minor = 0_i64;
        for (index, line) in input.lines.iter().enumerate() {
            require_text(&line.component_id)?;
            if line.quantity <= 0.0 || line.unit_price_minor < 0 {
                return Err(rusqlite::Error::InvalidQuery);
            }
            let (category, name, specification, unit, component_currency, brand, notes) =
                transaction.query_row(
                    "SELECT category, name, specification, unit, currency, brand, notes
                     FROM config_components WHERE id = ?1",
                    params![line.component_id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    },
                )?;
            if component_currency != currency {
                return Err(rusqlite::Error::InvalidQuery);
            }
            let amount_minor = (line.quantity * line.unit_price_minor as f64).round() as i64;
            total_amount_minor = total_amount_minor
                .checked_add(amount_minor)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            transaction.execute(
                "INSERT INTO configurable_product_lines(
                    id, configurable_product_id, sort_order, component_id, category_snapshot,
                    name_snapshot, specification_snapshot, quantity, unit_snapshot,
                    unit_price_minor, brand_snapshot, notes_snapshot, amount_minor
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    index as i64,
                    line.component_id,
                    category,
                    name,
                    specification,
                    line.quantity,
                    unit,
                    line.unit_price_minor,
                    brand,
                    notes,
                    amount_minor,
                ],
            )?;
        }
        transaction.execute(
            "UPDATE configurable_products SET total_amount_minor = ?2 WHERE id = ?1",
            params![id, total_amount_minor],
        )?;
        transaction.commit()?;
        self.audit("configurable_product", &id, "save")?;
        self.get_configurable_product(&id)
    }

    pub fn list_customers(&self) -> rusqlite::Result<Vec<Customer>> {
        let mut statement = self.connection.prepare(
            "SELECT id, code, legal_name, market, currency, payment_terms, address,
                    shipping_address, billing_address, purchase_intent, customer_analysis,
                    strengths, weaknesses, contacts, active
             FROM customers WHERE active = 1 ORDER BY code COLLATE NOCASE",
        )?;
        statement.query_map([], customer_from_row)?.collect()
    }

    pub fn save_customer(&self, input: CustomerInput) -> rusqlite::Result<Customer> {
        require_text(&input.code)?;
        require_text(&input.legal_name)?;
        require_text(&input.currency)?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO customers(
                id, code, legal_name, market, currency, payment_terms, address,
                shipping_address, billing_address, purchase_intent, customer_analysis,
                strengths, weaknesses, contacts, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 1)
             ON CONFLICT(id) DO UPDATE SET code = excluded.code, legal_name = excluded.legal_name,
                market = excluded.market, currency = excluded.currency,
                payment_terms = excluded.payment_terms, address = excluded.address,
                shipping_address = excluded.shipping_address, billing_address = excluded.billing_address,
                purchase_intent = excluded.purchase_intent, customer_analysis = excluded.customer_analysis,
                strengths = excluded.strengths, weaknesses = excluded.weaknesses,
                contacts = excluded.contacts, active = 1",
            params![
                id,
                input.code.trim(),
                input.legal_name.trim(),
                input.market.trim(),
                input.currency.trim().to_uppercase(),
                input.payment_terms.trim(),
                input.address.trim(),
                input.shipping_address.trim(),
                input.billing_address.trim(),
                input.purchase_intent.trim(),
                input.customer_analysis.trim(),
                input.strengths.trim(),
                input.weaknesses.trim(),
                input.contacts.trim()
            ],
        )?;
        self.audit("customer", &id, "save")?;
        self.connection.query_row(
            "SELECT id, code, legal_name, market, currency, payment_terms, address,
                    shipping_address, billing_address, purchase_intent, customer_analysis,
                    strengths, weaknesses, contacts, active
             FROM customers WHERE id = ?1",
            params![id],
            customer_from_row,
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
            "config_component" => "config_components",
            "configurable_product" => "configurable_products",
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

    pub fn list_documents(&self) -> rusqlite::Result<Vec<TradeDocument>> {
        let mut statement = self.connection.prepare(
            "SELECT id, document_type, number, trade_case_id, trade_case_number_snapshot,
                    customer_name_snapshot, version, status, language, issue_date, currency,
                    template_version, payload_json, void_reason, pdf_path, pdf_sha256,
                    exported_at, created_at, updated_at
             FROM documents ORDER BY updated_at DESC, number COLLATE NOCASE DESC, version DESC",
        )?;
        statement.query_map([], map_document)?.collect()
    }

    pub fn get_document(&self, id: &str) -> rusqlite::Result<TradeDocument> {
        self.connection.query_row(
            "SELECT id, document_type, number, trade_case_id, trade_case_number_snapshot,
                    customer_name_snapshot, version, status, language, issue_date, currency,
                    template_version, payload_json, void_reason, pdf_path, pdf_sha256,
                    exported_at, created_at, updated_at
             FROM documents WHERE id = ?1",
            params![id],
            map_document,
        )
    }

    pub fn create_document(&self, input: CreateDocumentInput) -> rusqlite::Result<TradeDocument> {
        require_text(&input.business_case_id)?;
        require_text(&input.number)?;
        require_text(&input.issue_date)?;
        validate_language(&input.language)?;
        let business_case = self.get_business_case(&input.business_case_id)?;
        let company_name = self
            .connection
            .query_row(
                "SELECT value FROM workspace_meta WHERE key = 'company_name'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(|| "Local Exporter".to_owned());
        let (destination_country, customer_address, shipping_address, billing_address) =
            self.connection.query_row(
                "SELECT market, address, shipping_address, billing_address
                 FROM customers WHERE id = ?1",
                params![business_case.customer_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )?;
        let preferred_buyer_address = match &input.document_type {
            DocumentType::PackingList => &shipping_address,
            DocumentType::CommercialInvoice | DocumentType::ProformaInvoice => &billing_address,
            _ => &customer_address,
        };
        let buyer_address = if preferred_buyer_address.trim().is_empty() {
            customer_address
        } else {
            preferred_buyer_address.clone()
        };
        let valid_until = self.connection.query_row(
            "SELECT date(?1, '+30 days')",
            params![input.issue_date.trim()],
            |row| row.get::<_, String>(0),
        )?;
        let mut lines = Vec::with_capacity(business_case.lines.len());
        for line in &business_case.lines {
            let (model, hs_code, gross_weight_kg) = self.connection.query_row(
                "SELECT model, hs_code, gross_weight_kg FROM products WHERE id = ?1",
                params![line.product_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )?;
            lines.push(DocumentLineSnapshot {
                product_id: line.product_id.clone(),
                sku: line.sku.clone(),
                description: if line.name_en.trim().is_empty() {
                    line.name_zh.clone()
                } else {
                    line.name_en.clone()
                },
                model,
                hs_code,
                quantity: line.quantity,
                unit: line.unit.clone(),
                unit_price_minor: line.unit_price_minor,
                amount_minor: line.amount_minor,
                packages: 1,
                package_type: "Carton".to_owned(),
                net_weight_kg: 0.0,
                gross_weight_kg: (gross_weight_kg * line.quantity * 1000.0).round() / 1000.0,
                cbm: 0.0,
            });
        }
        let payload = DocumentPayload {
            seller: company_name,
            seller_address: String::new(),
            buyer: business_case.customer_name.clone(),
            buyer_address,
            origin_country: "China".to_owned(),
            destination_country,
            port_of_loading: String::new(),
            port_of_discharge: String::new(),
            incoterm: business_case.incoterm.clone(),
            payment_terms: business_case.payment_terms.clone(),
            shipment_date: business_case.shipment_date.clone(),
            po_reference: business_case.number.clone(),
            valid_until,
            discount_minor: 0,
            bank_details: String::new(),
            notes: business_case.notes.clone(),
            declaration: "We certify that the information in this document is true and correct."
                .to_owned(),
            contract_terms: String::new(),
            lines,
        };
        let payload_json = serde_json::to_string(&payload).map_err(json_error)?;
        let id = Uuid::new_v4().to_string();
        self.connection.execute(
            "INSERT INTO documents(
                id, document_type, number, trade_case_id, trade_case_number_snapshot,
                customer_name_snapshot, version, status, language, issue_date, currency,
                template_version, payload_json
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 1, 'draft', ?7, ?8, ?9, 'base-1', ?10)",
            params![
                id,
                input.document_type.as_str(),
                input.number.trim(),
                business_case.id,
                business_case.number,
                business_case.customer_name,
                input.language.trim(),
                input.issue_date.trim(),
                business_case.currency,
                payload_json,
            ],
        )?;
        self.audit("document", &id, "create")?;
        self.get_document(&id)
    }

    pub fn convert_document(&self, input: ConvertDocumentInput) -> rusqlite::Result<TradeDocument> {
        require_text(&input.source_document_id)?;
        require_text(&input.number)?;
        require_text(&input.issue_date)?;
        validate_language(&input.language)?;
        let source = self.get_document(&input.source_document_id)?;
        if source.status != DocumentStatus::Issued
            || !matches!(
                (&source.document_type, &input.target_document_type),
                (
                    DocumentType::CommercialQuotation,
                    DocumentType::ProformaInvoice | DocumentType::TradeContract
                ) | (
                    DocumentType::ProformaInvoice,
                    DocumentType::TradeContract | DocumentType::CommercialInvoice
                )
            )
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = Uuid::new_v4().to_string();
        let payload_json = serde_json::to_string(&source.payload).map_err(json_error)?;
        self.connection.execute(
            "INSERT INTO documents(
                id, document_type, number, trade_case_id, trade_case_number_snapshot,
                customer_name_snapshot, version, status, language, issue_date, currency,
                template_version, payload_json
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 1, 'draft', ?7, ?8, ?9, 'base-1', ?10)",
            params![
                id,
                input.target_document_type.as_str(),
                input.number.trim(),
                source.business_case_id,
                source.business_case_number,
                source.customer_name,
                input.language.trim(),
                input.issue_date.trim(),
                source.currency,
                payload_json,
            ],
        )?;
        let audit_payload = serde_json::json!({
            "sourceDocumentId": input.source_document_id,
        })
        .to_string();
        self.connection.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('document', ?1, 'convert', ?2)",
            params![id, audit_payload],
        )?;
        self.get_document(&id)
    }

    pub fn save_document(&self, input: SaveDocumentInput) -> rusqlite::Result<TradeDocument> {
        require_text(&input.id)?;
        require_text(&input.number)?;
        require_text(&input.issue_date)?;
        validate_language(&input.language)?;
        let mut payload = input.payload;
        if payload.lines.is_empty() {
            return Err(rusqlite::Error::InvalidQuery);
        }
        for line in &mut payload.lines {
            if !line.quantity.is_finite()
                || line.quantity <= 0.0
                || line.unit_price_minor < 0
                || line.packages < 0
                || !line.net_weight_kg.is_finite()
                || !line.gross_weight_kg.is_finite()
                || !line.cbm.is_finite()
                || line.net_weight_kg < 0.0
                || line.gross_weight_kg < 0.0
                || line.cbm < 0.0
            {
                return Err(rusqlite::Error::InvalidQuery);
            }
            line.amount_minor = (line.quantity * line.unit_price_minor as f64).round() as i64;
        }
        let subtotal = payload
            .lines
            .iter()
            .map(|line| line.amount_minor)
            .sum::<i64>();
        if payload.discount_minor < 0 || payload.discount_minor > subtotal {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let payload_json = serde_json::to_string(&payload).map_err(json_error)?;
        let changed = self.connection.execute(
            "UPDATE documents SET number = ?1, language = ?2, issue_date = ?3,
                    payload_json = ?4, pdf_path = '', pdf_sha256 = '', exported_at = '',
                    updated_at = CURRENT_TIMESTAMP
             WHERE id = ?5 AND status = 'draft'",
            params![
                input.number.trim(),
                input.language.trim(),
                input.issue_date.trim(),
                payload_json,
                input.id,
            ],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        self.audit("document", &input.id, "save")?;
        self.get_document(&input.id)
    }

    pub fn issue_document(&self, id: &str) -> rusqlite::Result<TradeDocument> {
        let document = self.get_document(id)?;
        if document.status != DocumentStatus::Draft
            || crate::document::has_blocking_errors(&document)
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let mut checked_products = std::collections::HashSet::new();
        for line in &document.payload.lines {
            if !checked_products.insert(&line.product_id) {
                continue;
            }
            let case_quantity = self.connection.query_row(
                "SELECT COALESCE(SUM(quantity), 0) FROM trade_case_lines
                 WHERE trade_case_id = ?1 AND product_id = ?2",
                params![document.business_case_id, line.product_id],
                |row| row.get::<_, f64>(0),
            )?;
            let document_quantity = document
                .payload
                .lines
                .iter()
                .filter(|item| item.product_id == line.product_id)
                .map(|item| item.quantity)
                .sum::<f64>();
            if document_quantity > case_quantity + 0.000_001 {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "UPDATE documents SET status = 'issued', updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND status = 'draft'",
            params![id],
        )?;
        match document.document_type {
            DocumentType::CommercialQuotation => {}
            DocumentType::ProformaInvoice | DocumentType::TradeContract => {
                transaction.execute(
                    "UPDATE trade_cases SET stage = 'order'
                     WHERE id = ?1 AND stage = 'quotation'",
                    params![document.business_case_id],
                )?;
            }
            DocumentType::CommercialInvoice | DocumentType::PackingList => {
                transaction.execute(
                    "UPDATE trade_cases SET stage = 'documents' WHERE id = ?1",
                    params![document.business_case_id],
                )?;
            }
        }
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('document', ?1, 'issue', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.get_document(id)
    }

    pub fn void_document(&self, id: &str, reason: &str) -> rusqlite::Result<TradeDocument> {
        require_text(reason)?;
        let changed = self.connection.execute(
            "UPDATE documents SET status = 'voided', void_reason = ?1,
                    updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2 AND status = 'issued'",
            params![reason.trim(), id],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        self.audit("document", id, "void")?;
        self.get_document(id)
    }

    pub fn create_document_version(&self, id: &str) -> rusqlite::Result<TradeDocument> {
        let source = self.get_document(id)?;
        if source.status == DocumentStatus::Draft {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let next_version = self.connection.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM documents
             WHERE document_type = ?1 AND number = ?2",
            params![source.document_type.as_str(), source.number],
            |row| row.get::<_, i64>(0),
        )?;
        let new_id = Uuid::new_v4().to_string();
        let payload_json = serde_json::to_string(&source.payload).map_err(json_error)?;
        self.connection.execute(
            "INSERT INTO documents(
                id, document_type, number, trade_case_id, trade_case_number_snapshot,
                customer_name_snapshot, version, status, language, issue_date, currency,
                template_version, payload_json
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, 'draft', ?8, ?9, ?10, ?11, ?12)",
            params![
                new_id,
                source.document_type.as_str(),
                source.number,
                source.business_case_id,
                source.business_case_number,
                source.customer_name,
                next_version,
                source.language,
                source.issue_date,
                source.currency,
                source.template_version,
                payload_json,
            ],
        )?;
        self.audit("document", &new_id, "new_version")?;
        self.get_document(&new_id)
    }

    pub fn update_document_export(
        &self,
        id: &str,
        path: &str,
        sha256: &str,
    ) -> rusqlite::Result<TradeDocument> {
        let changed = self.connection.execute(
            "UPDATE documents SET pdf_path = ?1, pdf_sha256 = ?2,
                    exported_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![path, sha256, id],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.audit("document", id, "export_pdf")?;
        self.get_document(id)
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
            "documents" => "SELECT COUNT(*) FROM documents WHERE status <> 'voided'",
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        let count = self
            .connection
            .query_row(query, [], |row| row.get::<_, i64>(0))?;
        Ok(count as u64)
    }
}

fn map_config_component(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConfigComponent> {
    Ok(ConfigComponent {
        id: row.get(0)?,
        code: row.get(1)?,
        category: row.get(2)?,
        name: row.get(3)?,
        specification: row.get(4)?,
        default_quantity: row.get(5)?,
        unit: row.get(6)?,
        unit_price_minor: row.get(7)?,
        currency: row.get(8)?,
        brand: row.get(9)?,
        notes: row.get(10)?,
        active: row.get::<_, i64>(11)? != 0,
    })
}

fn map_configurable_product_line(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<ConfigurableProductLine> {
    Ok(ConfigurableProductLine {
        id: row.get(0)?,
        component_id: row.get(1)?,
        category: row.get(2)?,
        name: row.get(3)?,
        specification: row.get(4)?,
        quantity: row.get(5)?,
        unit: row.get(6)?,
        unit_price_minor: row.get(7)?,
        brand: row.get(8)?,
        notes: row.get(9)?,
        amount_minor: row.get(10)?,
    })
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

fn map_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<TradeDocument> {
    let type_value: String = row.get(1)?;
    let status_value: String = row.get(7)?;
    let payload_value: String = row.get(12)?;
    let document_type = DocumentType::from_db(&type_value).ok_or(rusqlite::Error::InvalidQuery)?;
    let status = DocumentStatus::from_db(&status_value).ok_or(rusqlite::Error::InvalidQuery)?;
    let payload = serde_json::from_str::<DocumentPayload>(&payload_value).map_err(json_error)?;
    let mut document = TradeDocument {
        id: row.get(0)?,
        document_type,
        number: row.get(2)?,
        business_case_id: row.get(3)?,
        business_case_number: row.get(4)?,
        customer_name: row.get(5)?,
        version: row.get::<_, i64>(6)? as u32,
        status,
        language: row.get(8)?,
        issue_date: row.get(9)?,
        currency: row.get(10)?,
        template_version: row.get(11)?,
        payload,
        validation_issues: Vec::new(),
        void_reason: row.get(13)?,
        pdf_path: row.get(14)?,
        pdf_sha256: row.get(15)?,
        exported_at: row.get(16)?,
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
    };
    document.validation_issues = crate::document::validate(&document);
    Ok(document)
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

fn customer_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: row.get(0)?,
        code: row.get(1)?,
        legal_name: row.get(2)?,
        market: row.get(3)?,
        currency: row.get(4)?,
        payment_terms: row.get(5)?,
        address: row.get(6)?,
        shipping_address: row.get(7)?,
        billing_address: row.get(8)?,
        purchase_intent: row.get(9)?,
        customer_analysis: row.get(10)?,
        strengths: row.get(11)?,
        weaknesses: row.get(12)?,
        contacts: row.get(13)?,
        active: row.get::<_, i64>(14)? != 0,
    })
}

fn validate_language(value: &str) -> rusqlite::Result<()> {
    if matches!(value.trim(), "en" | "zh_en" | "ru") {
        Ok(())
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

fn json_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BusinessCaseLineInput, ConfigurableProductLineInput};

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
            let oil_tank = database
                .save_config_component(ConfigComponentInput {
                    id: None,
                    code: "COMP-OIL-200".into(),
                    category: "润滑油系统".into(),
                    name: "润滑油补给箱".into(),
                    specification: "V=200L碳钢补油箱".into(),
                    default_quantity: 1.0,
                    unit: "套".into(),
                    unit_price_minor: 545_400,
                    currency: "CNY".into(),
                    brand: "康达".into(),
                    notes: String::new(),
                })
                .unwrap();
            let silencer = database
                .save_config_component(ConfigComponentInput {
                    id: None,
                    code: "COMP-EXHAUST-01".into(),
                    category: "排气系统".into(),
                    name: "消声器".into(),
                    specification: "碳钢材质，含法兰".into(),
                    default_quantity: 2.0,
                    unit: "只".into(),
                    unit_price_minor: 350_000,
                    currency: "CNY".into(),
                    brand: "康达".into(),
                    notes: String::new(),
                })
                .unwrap();
            let configured = database
                .save_configurable_product(ConfigurableProductInput {
                    id: None,
                    code: "CFG-K38-G6".into(),
                    name: "600KW天然气发电机组".into(),
                    model: "K38N-G6".into(),
                    currency: "CNY".into(),
                    notes: "配置报价测试".into(),
                    lines: vec![
                        ConfigurableProductLineInput {
                            component_id: oil_tank.id,
                            quantity: 1.0,
                            unit_price_minor: 545_400,
                        },
                        ConfigurableProductLineInput {
                            component_id: silencer.id,
                            quantity: 2.0,
                            unit_price_minor: 350_000,
                        },
                    ],
                })
                .unwrap();
            assert_eq!(configured.lines.len(), 2);
            assert_eq!(configured.total_amount_minor, 1_245_400);
            let customer = database
                .save_customer(CustomerInput {
                    id: None,
                    code: "CUS-1".into(),
                    legal_name: "Example Import LLC".into(),
                    market: "US".into(),
                    currency: "USD".into(),
                    payment_terms: "30% deposit".into(),
                    address: "Seattle, USA".into(),
                    shipping_address: "Port of Seattle".into(),
                    billing_address: "Seattle, USA".into(),
                    purchase_intent: "Annual container orders".into(),
                    customer_analysis: "Growing regional importer".into(),
                    strengths: "Stable channel".into(),
                    weaknesses: "Long approval cycle".into(),
                    contacts: "Jane Smith | Purchasing | jane@example.com".into(),
                })
                .unwrap();
            assert_eq!(customer.shipping_address, "Port of Seattle");
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
                    stage: PipelineStage::Quotation,
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
            let quote = database
                .create_document(CreateDocumentInput {
                    business_case_id: business_case.id.clone(),
                    document_type: DocumentType::CommercialQuotation,
                    number: "QUO-2026-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-06".into(),
                })
                .unwrap();
            assert_eq!(quote.payload.valid_until, "2026-09-05");
            assert_eq!(quote.payload.buyer_address, "Seattle, USA");
            let mut quote_payload = quote.payload.clone();
            quote_payload.seller_address = "Shenzhen, China".into();
            quote_payload.buyer_address = "Seattle, USA".into();
            quote_payload.discount_minor = 100;
            let quote = database
                .save_document(SaveDocumentInput {
                    id: quote.id,
                    number: quote.number,
                    language: quote.language,
                    issue_date: quote.issue_date,
                    payload: quote_payload,
                })
                .unwrap();
            let issued_quote = database.issue_document(&quote.id).unwrap();
            let proforma = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued_quote.id.clone(),
                    target_document_type: DocumentType::ProformaInvoice,
                    number: "PI-2026-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-10-07".into(),
                })
                .unwrap();
            assert_eq!(proforma.payload.discount_minor, 100);
            assert_eq!(proforma.payload.seller_address, "Shenzhen, China");
            let issued_proforma = database.issue_document(&proforma.id).unwrap();
            assert_eq!(
                database.get_business_case(&business_case.id).unwrap().stage,
                PipelineStage::Order
            );
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
            let draft = database
                .create_document(CreateDocumentInput {
                    business_case_id: business_case.id.clone(),
                    document_type: DocumentType::CommercialInvoice,
                    number: "INV-2026-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-06".into(),
                })
                .unwrap();
            assert_eq!(draft.status, DocumentStatus::Draft);
            assert_eq!(draft.payload.lines.len(), 1);
            assert_eq!(draft.payload.lines[0].amount_minor, 3_000);
            let mut payload = draft.payload.clone();
            payload.seller_address = "Shenzhen, China".into();
            payload.buyer_address = "Seattle, USA".into();
            let saved = database
                .save_document(SaveDocumentInput {
                    id: draft.id.clone(),
                    number: draft.number.clone(),
                    language: draft.language.clone(),
                    issue_date: draft.issue_date.clone(),
                    payload: payload.clone(),
                })
                .unwrap();
            assert!(
                saved
                    .validation_issues
                    .iter()
                    .all(|issue| { issue.severity != crate::domain::ValidationSeverity::Error })
            );
            let issued = database.issue_document(&saved.id).unwrap();
            assert_eq!(issued.status, DocumentStatus::Issued);
            if let Some(typst) = crate::document::find_typst(std::path::Path::new("")) {
                let render_root =
                    std::env::temp_dir().join(format!("tradedesk-pdf-{}", Uuid::new_v4()));
                let work_dir = render_root.join("work");
                let output_dir = render_root.join("output");
                let export =
                    crate::document::export_pdf(&issued, &typst, &work_dir, &output_dir).unwrap();
                let pdf = std::fs::read(&export.path).unwrap();
                assert_eq!(&pdf[..5], b"%PDF-");
                assert_eq!(export.sha256.len(), 64);
                let csv = crate::document::export_csv(&issued, &output_dir).unwrap();
                assert!(std::fs::read_to_string(csv).unwrap().contains("SKU-1"));
                for sales_document in [&issued_quote, &issued_proforma] {
                    let sales_export =
                        crate::document::export_pdf(sales_document, &typst, &work_dir, &output_dir)
                            .unwrap();
                    assert_eq!(&std::fs::read(sales_export.path).unwrap()[..5], b"%PDF-");
                }
                let _ = std::fs::remove_dir_all(render_root);
            }
            assert!(
                database
                    .save_document(SaveDocumentInput {
                        id: issued.id.clone(),
                        number: issued.number.clone(),
                        language: issued.language.clone(),
                        issue_date: issued.issue_date.clone(),
                        payload,
                    })
                    .is_err()
            );
            let version_two = database.create_document_version(&issued.id).unwrap();
            assert_eq!(version_two.version, 2);
            assert_eq!(version_two.status, DocumentStatus::Draft);
            let voided = database
                .void_document(&issued.id, "Replaced by corrected version")
                .unwrap();
            assert_eq!(voided.status, DocumentStatus::Voided);
            assert_eq!(database.summary().unwrap().products, 1);
            assert_eq!(database.summary().unwrap().active_cases, 1);
            assert_eq!(database.summary().unwrap().purchase_orders, 1);
            assert_eq!(database.summary().unwrap().documents, 3);
        }
        let reopened =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        assert_eq!(reopened.list_products().unwrap()[0].sku, "SKU-1");
        assert_eq!(
            reopened.list_configurable_products().unwrap()[0].total_amount_minor,
            1_245_400
        );
        let business_cases = reopened.list_business_cases().unwrap();
        assert_eq!(business_cases[0].number, "TD-2026-0001");
        assert_eq!(business_cases[0].lines[0].name_en, "Test product");
        let purchase_orders = reopened.list_purchase_orders().unwrap();
        assert_eq!(purchase_orders[0].number, "PO-2026-0001");
        assert_eq!(purchase_orders[0].ready_quantity, 12.5);
        let documents = reopened.list_documents().unwrap();
        assert_eq!(documents.len(), 4);
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::CommercialInvoice && document.version == 2
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::CommercialInvoice
                && document.status == DocumentStatus::Voided
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::ProformaInvoice
                && document.status == DocumentStatus::Issued
        }));
        drop(reopened);

        let header = std::fs::read(&path).unwrap();
        assert_ne!(&header[..16], b"SQLite format 3\0");
        assert!(
            EncryptedDatabase::open(&path, Zeroizing::new("wrong-password".to_owned())).is_err()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn migrates_customer_profiles_from_schema_v5() {
        let path = std::env::temp_dir().join(format!("tradedesk-v5-{}.db", Uuid::new_v4()));
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .pragma_update(None, "key", "test-password")
                .unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE workspace_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                     INSERT INTO workspace_meta(key, value) VALUES('schema_version', '5');
                     CREATE TABLE customers (
                        id TEXT PRIMARY KEY, code TEXT NOT NULL UNIQUE, legal_name TEXT NOT NULL,
                        market TEXT NOT NULL DEFAULT '', currency TEXT NOT NULL DEFAULT 'USD',
                        payment_terms TEXT NOT NULL DEFAULT '', active INTEGER NOT NULL DEFAULT 1
                     );
                     INSERT INTO customers(id, code, legal_name, market, currency, payment_terms, active)
                     VALUES('customer-1', 'CUS-1', 'Legacy Customer', 'US', 'USD', 'T/T', 1);",
                )
                .unwrap();
        }

        let database =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        let customers = database.list_customers().unwrap();
        assert_eq!(customers.len(), 1);
        assert_eq!(customers[0].legal_name, "Legacy Customer");
        assert!(customers[0].shipping_address.is_empty());
        assert!(customers[0].contacts.is_empty());
        assert_eq!(
            database
                .connection
                .query_row(
                    "SELECT value FROM workspace_meta WHERE key = 'schema_version'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "7"
        );
        drop(database);
        let _ = std::fs::remove_file(&path);
    }
}
