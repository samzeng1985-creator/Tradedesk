use std::{path::Path, time::Duration};

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::domain::{
    AttachmentInput, AttachmentRecord, BusinessCase, BusinessCaseInput, BusinessCaseLine,
    CompanyProfile, CompanyRecord, CompanyRegistry, CompanySigningAsset, ComponentOption,
    ComponentOptionInput, ComponentOptionTranslationInput, ConfigComponent, ConfigComponentInput,
    ConfigurableProduct, ConfigurableProductInput, ConfigurableProductLine, ConvertDocumentInput,
    CostEstimate, CostEstimateInput, CostEstimateLine, CreateDocumentInput, Customer,
    CustomerInput, DocumentDraft, DocumentLineSnapshot, DocumentPayload, DocumentStatus,
    DocumentType, MilestoneStatus, Partner, PartnerInput, PaymentPlan, PaymentPlanInput,
    PaymentStatus, PipelineStage, Product, ProductInput, ProductionMilestone,
    ProductionMilestoneInput, PurchaseOrder, PurchaseOrderInput, PurchaseOrderLine, PurchaseStatus,
    SaveDocumentInput, ShipmentBatch, ShipmentBatchInput, ShipmentLine, ShipmentStatus, Supplier,
    SupplierInput, TradeDocument, WorkspaceSummary,
};

const SCHEMA_VERSION: i64 = 13;

const MILESTONE_STAGES: [(&str, &str); 6] = [
    ("raw_material", "原料准备"),
    ("started", "开工"),
    ("production", "生产完成"),
    ("quality", "质检"),
    ("packing", "包装"),
    ("ready_to_ship", "可发货"),
];

const COST_CATEGORIES: [&str; 10] = [
    "material",
    "processing",
    "packaging",
    "domestic_logistics",
    "international_freight",
    "duty_tax",
    "commission",
    "insurance",
    "certification",
    "other",
];

pub struct EncryptedDatabase {
    connection: Connection,
    key: Zeroizing<String>,
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

        let database = Self { connection, key };
        database.migrate()?;
        Ok(database)
    }

    pub fn recovery_secret(&self) -> Zeroizing<String> {
        Zeroizing::new(self.key.to_string())
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

             CREATE TABLE IF NOT EXISTS component_options (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                value TEXT NOT NULL COLLATE NOCASE,
                active INTEGER NOT NULL DEFAULT 1,
                UNIQUE(kind, value)
             );
             CREATE INDEX IF NOT EXISTS idx_component_options_kind
                ON component_options(kind, value COLLATE NOCASE);

             CREATE TABLE IF NOT EXISTS component_option_translations (
                option_id TEXT NOT NULL REFERENCES component_options(id) ON DELETE CASCADE,
                language TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY(option_id, language)
             );

             CREATE TABLE IF NOT EXISTS configurable_products (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                model TEXT NOT NULL DEFAULT '',
                currency TEXT NOT NULL DEFAULT 'CNY',
                exchange_rate REAL NOT NULL DEFAULT 1,
                exchange_rate_date TEXT NOT NULL DEFAULT '',
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
             );

             CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL DEFAULT '',
                file_name TEXT NOT NULL,
                mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
                content BLOB NOT NULL,
                size_bytes INTEGER NOT NULL,
                sha256 TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );
             CREATE INDEX IF NOT EXISTS idx_attachments_entity
                ON attachments(entity_type, entity_id, created_at DESC);

             CREATE TABLE IF NOT EXISTS drafts (
                draft_key TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
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
            "products",
            "record_type",
            "TEXT NOT NULL DEFAULT 'standard'",
        )?;
        ensure_column(
            &transaction,
            "configurable_products",
            "exchange_rate",
            "REAL NOT NULL DEFAULT 1",
        )?;
        ensure_column(
            &transaction,
            "configurable_products",
            "exchange_rate_date",
            "TEXT NOT NULL DEFAULT ''",
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
                ON documents(document_type, number, version DESC);

             CREATE TABLE IF NOT EXISTS partners (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                legal_name TEXT NOT NULL,
                partner_type TEXT NOT NULL DEFAULT 'freight_forwarder',
                contact TEXT NOT NULL DEFAULT '',
                address TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1
             );

             CREATE TABLE IF NOT EXISTS shipment_batches (
                id TEXT PRIMARY KEY,
                number TEXT NOT NULL UNIQUE,
                trade_case_id TEXT NOT NULL REFERENCES trade_cases(id),
                trade_case_number_snapshot TEXT NOT NULL,
                partner_id TEXT REFERENCES partners(id),
                partner_name_snapshot TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'planned',
                planned_date TEXT NOT NULL DEFAULT '',
                actual_date TEXT NOT NULL DEFAULT '',
                tracking_number TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1
             );
             CREATE INDEX IF NOT EXISTS idx_shipment_batches_case
                ON shipment_batches(trade_case_id, active, planned_date);

             CREATE TABLE IF NOT EXISTS shipment_batch_lines (
                id TEXT PRIMARY KEY,
                shipment_batch_id TEXT NOT NULL REFERENCES shipment_batches(id) ON DELETE CASCADE,
                trade_case_line_id TEXT NOT NULL REFERENCES trade_case_lines(id),
                sort_order INTEGER NOT NULL,
                sku_snapshot TEXT NOT NULL,
                product_name_snapshot TEXT NOT NULL,
                quantity REAL NOT NULL,
                unit_snapshot TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_shipment_batch_lines_batch
                ON shipment_batch_lines(shipment_batch_id, sort_order);
             CREATE INDEX IF NOT EXISTS idx_shipment_batch_lines_case_line
                ON shipment_batch_lines(trade_case_line_id);

             CREATE TABLE IF NOT EXISTS payment_plans (
                id TEXT PRIMARY KEY,
                number TEXT NOT NULL UNIQUE,
                trade_case_id TEXT NOT NULL REFERENCES trade_cases(id),
                trade_case_number_snapshot TEXT NOT NULL,
                payment_type TEXT NOT NULL DEFAULT 'deposit',
                due_date TEXT NOT NULL DEFAULT '',
                currency TEXT NOT NULL,
                amount_minor INTEGER NOT NULL,
                received_amount_minor INTEGER NOT NULL DEFAULT 0,
                received_date TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'planned',
                notes TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1
             );
             CREATE INDEX IF NOT EXISTS idx_payment_plans_case
                ON payment_plans(trade_case_id, active, due_date);

             CREATE TABLE IF NOT EXISTS cost_estimates (
                id TEXT PRIMARY KEY,
                number TEXT NOT NULL UNIQUE,
                trade_case_id TEXT NOT NULL REFERENCES trade_cases(id),
                trade_case_number_snapshot TEXT NOT NULL,
                customer_name_snapshot TEXT NOT NULL,
                currency TEXT NOT NULL,
                target_margin_bps INTEGER NOT NULL DEFAULT 2500,
                notes TEXT NOT NULL DEFAULT '',
                total_cost_minor INTEGER NOT NULL DEFAULT 0,
                suggested_price_minor INTEGER NOT NULL DEFAULT 0,
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );
             CREATE INDEX IF NOT EXISTS idx_cost_estimates_case
                ON cost_estimates(trade_case_id, active, updated_at DESC);

             CREATE TABLE IF NOT EXISTS cost_estimate_lines (
                id TEXT PRIMARY KEY,
                cost_estimate_id TEXT NOT NULL REFERENCES cost_estimates(id) ON DELETE CASCADE,
                sort_order INTEGER NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                specification TEXT NOT NULL DEFAULT '',
                quantity REAL NOT NULL,
                unit TEXT NOT NULL,
                unit_cost_minor INTEGER NOT NULL,
                amount_minor INTEGER NOT NULL,
                notes TEXT NOT NULL DEFAULT ''
             );
             CREATE INDEX IF NOT EXISTS idx_cost_estimate_lines_estimate
                ON cost_estimate_lines(cost_estimate_id, sort_order);",
        )?;

        ensure_column(
            &transaction,
            "attachments",
            "entity_label",
            "TEXT NOT NULL DEFAULT ''",
        )?;

        ensure_column(
            &transaction,
            "trade_case_lines",
            "source_type",
            "TEXT NOT NULL DEFAULT 'product'",
        )?;

        transaction.execute_batch(
            "INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'category', trim(category), 1
                FROM config_components WHERE trim(category) <> '' GROUP BY trim(category);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'name', trim(name), 1
                FROM config_components WHERE trim(name) <> '' GROUP BY trim(name);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'brand', trim(brand), 1
                FROM config_components WHERE trim(brand) <> '' GROUP BY trim(brand);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'specification', trim(specification), 1
                FROM config_components WHERE trim(specification) <> '' GROUP BY trim(specification);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'unit', trim(unit), 1
                FROM config_components WHERE trim(unit) <> '' GROUP BY trim(unit);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'notes', trim(notes), 1
                FROM config_components WHERE trim(notes) <> '' GROUP BY trim(notes);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'product_name', trim(name), 1
                FROM configurable_products WHERE trim(name) <> '' GROUP BY trim(name);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'configuration_notes', trim(notes), 1
                FROM configurable_products WHERE trim(notes) <> '' GROUP BY trim(notes);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'category', trim(category_snapshot), 1
                FROM configurable_product_lines WHERE trim(category_snapshot) <> '' GROUP BY trim(category_snapshot);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'name', trim(name_snapshot), 1
                FROM configurable_product_lines WHERE trim(name_snapshot) <> '' GROUP BY trim(name_snapshot);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'specification', trim(specification_snapshot), 1
                FROM configurable_product_lines WHERE trim(specification_snapshot) <> '' GROUP BY trim(specification_snapshot);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'unit', trim(unit_snapshot), 1
                FROM configurable_product_lines WHERE trim(unit_snapshot) <> '' GROUP BY trim(unit_snapshot);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'brand', trim(brand_snapshot), 1
                FROM configurable_product_lines WHERE trim(brand_snapshot) <> '' GROUP BY trim(brand_snapshot);
             INSERT OR IGNORE INTO component_options(id, kind, value, active)
                SELECT lower(hex(randomblob(16))), 'notes', trim(notes_snapshot), 1
                FROM configurable_product_lines WHERE trim(notes_snapshot) <> '' GROUP BY trim(notes_snapshot);
             INSERT OR IGNORE INTO products(
                id, sku, name_zh, name_en, model, hs_code, unit, gross_weight_kg, active, record_type
             ) SELECT id, '@CFG:' || id, name, name, model, '', '套', 0, active, 'configurable'
               FROM configurable_products;",
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

    pub fn company_registry(&self) -> rusqlite::Result<CompanyRegistry> {
        let value = |key: &str| -> rusqlite::Result<String> {
            Ok(self
                .connection
                .query_row(
                    "SELECT value FROM workspace_meta WHERE key = ?1",
                    params![key],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .unwrap_or_default())
        };
        let saved = value("company_registry_json")?;
        if !saved.is_empty() {
            return serde_json::from_str(&saved).map_err(json_error);
        }
        let company_name = value("company_name")?;
        let signature_data_url = value("company_signature_data_url")?;
        let signing_assets = if signature_data_url.is_empty() {
            Vec::new()
        } else {
            vec![CompanySigningAsset {
                id: "legacy-signature".to_owned(),
                name: "默认电子签名".to_owned(),
                kind: "signature".to_owned(),
                data_url: signature_data_url,
            }]
        };
        Ok(CompanyRegistry {
            default_company_id: "company-default".to_owned(),
            companies: vec![CompanyRecord {
                id: "company-default".to_owned(),
                company_name: if company_name.trim().is_empty() {
                    "本地工作区".to_owned()
                } else {
                    company_name
                },
                logo_data_url: value("company_logo_data_url")?,
                signing_assets,
            }],
        })
    }

    pub fn save_company_registry(
        &self,
        input: CompanyRegistry,
    ) -> rusqlite::Result<CompanyRegistry> {
        let default_company = input
            .companies
            .iter()
            .find(|company| company.id == input.default_company_id)
            .ok_or(rusqlite::Error::InvalidQuery)?;
        let encoded = serde_json::to_string(&input).map_err(json_error)?;
        let transaction = self.connection.unchecked_transaction()?;
        for (key, value) in [
            ("company_name", default_company.company_name.trim()),
            ("company_registry_json", encoded.as_str()),
        ] {
            transaction.execute(
                "INSERT INTO workspace_meta(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
        }
        transaction.commit()?;
        self.audit("workspace", "company_registry", "update")?;
        self.company_registry()
    }

    pub fn resolve_company_profile(
        &self,
        company_id: &str,
        signing_asset_id: &str,
    ) -> rusqlite::Result<CompanyProfile> {
        let registry = self.company_registry()?;
        let selected_company_id = if company_id.trim().is_empty() {
            registry.default_company_id.as_str()
        } else {
            company_id
        };
        let company = registry
            .companies
            .iter()
            .find(|company| company.id == selected_company_id)
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let asset = if signing_asset_id.trim().is_empty() {
            None
        } else {
            Some(
                company
                    .signing_assets
                    .iter()
                    .find(|asset| asset.id == signing_asset_id)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?,
            )
        };
        Ok(CompanyProfile {
            company_name: company.company_name.clone(),
            logo_data_url: company.logo_data_url.clone(),
            signature_data_url: asset
                .map(|asset| asset.data_url.clone())
                .unwrap_or_default(),
            signing_asset_kind: asset.map(|asset| asset.kind.clone()).unwrap_or_default(),
        })
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
            recovery_key: String::new(),
            recovery_ready: false,
        })
    }

    pub fn list_products(&self) -> rusqlite::Result<Vec<Product>> {
        let mut statement = self.connection.prepare(
            "SELECT id, sku, name_zh, name_en, model, hs_code, unit, gross_weight_kg, active
             FROM products WHERE active = 1 AND record_type = 'standard'
             ORDER BY sku COLLATE NOCASE",
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

    pub fn list_component_options(&self) -> rusqlite::Result<Vec<ComponentOption>> {
        let mut options = {
            let mut statement = self.connection.prepare(
                "SELECT id, kind, value, active FROM component_options
                 WHERE active = 1 ORDER BY kind, value COLLATE NOCASE",
            )?;
            statement
                .query_map([], |row| {
                    Ok(ComponentOption {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        value: row.get(2)?,
                        active: row.get::<_, i64>(3)? != 0,
                        translations: std::collections::BTreeMap::new(),
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for option in &mut options {
            let mut statement = self.connection.prepare(
                "SELECT language, value FROM component_option_translations
                 WHERE option_id = ?1 ORDER BY language",
            )?;
            option.translations = statement
                .query_map(params![option.id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<std::collections::BTreeMap<_, _>>>()?;
        }
        Ok(options)
    }

    pub fn save_component_option(
        &self,
        input: ComponentOptionInput,
    ) -> rusqlite::Result<ComponentOption> {
        validate_component_option_kind(&input.kind)?;
        require_text(&input.value)?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO component_options(id, kind, value, active)
             VALUES(?1, ?2, ?3, 1)
             ON CONFLICT(kind, value) DO UPDATE SET active = 1",
            params![id, input.kind, input.value.trim()],
        )?;
        let option = self.connection.query_row(
            "SELECT id, kind, value, active FROM component_options
             WHERE kind = ?1 AND value = ?2 COLLATE NOCASE",
            params![input.kind, input.value.trim()],
            |row| {
                Ok(ComponentOption {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    value: row.get(2)?,
                    active: row.get::<_, i64>(3)? != 0,
                    translations: std::collections::BTreeMap::new(),
                })
            },
        )?;
        self.audit("component_option", &option.id, "save")?;
        Ok(option)
    }

    pub fn save_component_option_translation(
        &self,
        input: ComponentOptionTranslationInput,
    ) -> rusqlite::Result<ComponentOption> {
        require_text(&input.option_id)?;
        validate_configuration_language(&input.language)?;
        require_text(&input.value)?;
        self.connection.execute(
            "INSERT INTO component_option_translations(option_id, language, value)
             VALUES(?1, ?2, ?3)
             ON CONFLICT(option_id, language) DO UPDATE SET value = excluded.value",
            params![input.option_id, input.language, input.value.trim()],
        )?;
        self.audit("component_option", &input.option_id, "translate")?;
        self.list_component_options()?
            .into_iter()
            .find(|option| option.id == input.option_id)
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
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
        self.remember_component_option("category", &input.category)?;
        self.remember_component_option("name", &input.name)?;
        self.remember_component_option("brand", &input.brand)?;
        self.remember_component_option("specification", &input.specification)?;
        self.remember_component_option("unit", &input.unit)?;
        self.remember_component_option("notes", &input.notes)?;
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
            "SELECT id, code, name, model, currency, exchange_rate, exchange_rate_date,
                    notes, total_amount_minor, active
             FROM configurable_products WHERE id = ?1",
            params![id],
            |row| {
                Ok(ConfigurableProduct {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    model: row.get(3)?,
                    currency: row.get(4)?,
                    exchange_rate: row.get(5)?,
                    exchange_rate_date: row.get(6)?,
                    notes: row.get(7)?,
                    total_amount_minor: row.get(8)?,
                    active: row.get::<_, i64>(9)? != 0,
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

    pub fn configuration_for_export(
        &self,
        id: &str,
        language: &str,
    ) -> rusqlite::Result<(ConfigurableProduct, Vec<String>)> {
        validate_configuration_language(language)?;
        let mut configuration = self.get_configurable_product(id)?;
        let mut missing = std::collections::BTreeSet::new();
        configuration.name = self.localized_value(
            "product_name",
            &configuration.name,
            language,
            "产品名称",
            &mut missing,
        )?;
        configuration.notes = self.localized_value(
            "configuration_notes",
            &configuration.notes,
            language,
            "配置说明",
            &mut missing,
        )?;
        for line in &mut configuration.lines {
            line.category = self.localized_value(
                "category",
                &line.category,
                language,
                "组件类别",
                &mut missing,
            )?;
            line.name = self.localized_value("name", &line.name, language, "品名", &mut missing)?;
            line.specification = self.localized_value(
                "specification",
                &line.specification,
                language,
                "型号/规格/材质",
                &mut missing,
            )?;
            line.unit = self.localized_value("unit", &line.unit, language, "单位", &mut missing)?;
            line.brand =
                self.localized_value("brand", &line.brand, language, "品牌", &mut missing)?;
            line.notes =
                self.localized_value("notes", &line.notes, language, "备注", &mut missing)?;
        }
        Ok((configuration, missing.into_iter().collect()))
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
        let exchange_rate = if currency == "CNY" {
            1.0
        } else {
            input.exchange_rate
        };
        let exchange_rate_date = if currency == "CNY" {
            String::new()
        } else {
            require_text(&input.exchange_rate_date)?;
            input.exchange_rate_date.trim().to_owned()
        };
        if !exchange_rate.is_finite() || exchange_rate <= 0.0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT INTO configurable_products(
                id, code, name, model, currency, exchange_rate, exchange_rate_date,
                notes, total_amount_minor, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 1)
             ON CONFLICT(id) DO UPDATE SET code = excluded.code, name = excluded.name,
                model = excluded.model, currency = excluded.currency,
                exchange_rate = excluded.exchange_rate,
                exchange_rate_date = excluded.exchange_rate_date,
                notes = excluded.notes,
                active = 1",
            params![
                id,
                input.code.trim(),
                input.name.trim(),
                input.model.trim(),
                currency,
                exchange_rate,
                exchange_rate_date,
                input.notes.trim(),
            ],
        )?;
        transaction.execute(
            "INSERT INTO products(
                id, sku, name_zh, name_en, model, hs_code, unit,
                gross_weight_kg, active, record_type
             ) VALUES(?1, ?2, ?3, ?3, ?4, '', '套', 0, 1, 'configurable')
             ON CONFLICT(id) DO UPDATE SET name_zh = excluded.name_zh,
                name_en = excluded.name_en, model = excluded.model,
                active = 1, record_type = 'configurable'",
            params![
                id,
                format!("@CFG:{id}"),
                input.name.trim(),
                input.model.trim()
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
            if component_currency != currency && component_currency != "CNY" {
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
        self.remember_component_option("product_name", &input.name)?;
        self.remember_component_option("configuration_notes", &input.notes)?;
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
            "component_option" => "component_options",
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
        if entity == "configurable_product" {
            self.connection.execute(
                "UPDATE products SET active = 0 WHERE id = ?1 AND record_type = 'configurable'",
                params![id],
            )?;
        }
        self.audit(entity, id, "archive")
    }

    pub fn master_record_id(&self, entity: &str, code: &str) -> rusqlite::Result<Option<String>> {
        let (table, column) = match entity {
            "product" => ("products", "sku"),
            "customer" => ("customers", "code"),
            "supplier" => ("suppliers", "code"),
            "config_component" => ("config_components", "code"),
            "configurable_product" => ("configurable_products", "code"),
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        self.connection
            .query_row(
                &format!("SELECT id FROM {table} WHERE {column} = ?1 COLLATE NOCASE"),
                params![code.trim()],
                |row| row.get(0),
            )
            .optional()
    }

    fn remember_component_option(&self, kind: &str, value: &str) -> rusqlite::Result<()> {
        if value.trim().is_empty() {
            return Ok(());
        }
        validate_component_option_kind(kind)?;
        self.connection.execute(
            "INSERT INTO component_options(id, kind, value, active)
             VALUES(?1, ?2, ?3, 1)
             ON CONFLICT(kind, value) DO UPDATE SET active = 1",
            params![Uuid::new_v4().to_string(), kind, value.trim()],
        )?;
        Ok(())
    }

    fn localized_value(
        &self,
        kind: &str,
        source: &str,
        language: &str,
        label: &str,
        missing: &mut std::collections::BTreeSet<String>,
    ) -> rusqlite::Result<String> {
        if source.trim().is_empty() || !contains_cjk(source) {
            return Ok(source.to_owned());
        }
        let translated = self
            .connection
            .query_row(
                "SELECT translation.value
                 FROM component_options option
                 JOIN component_option_translations translation ON translation.option_id = option.id
                 WHERE option.kind = ?1 AND option.value = ?2 COLLATE NOCASE
                   AND translation.language = ?3",
                params![kind, source.trim(), language],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if let Some(value) = translated {
            Ok(value)
        } else {
            missing.insert(format!("{label}：{}", source.trim()));
            Ok(source.to_owned())
        }
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
            "SELECT id, source_type, product_id, sku_snapshot, name_zh_snapshot, name_en_snapshot,
                    quantity, unit_snapshot, unit_price_minor, amount_minor
             FROM trade_case_lines WHERE trade_case_id = ?1 ORDER BY sort_order",
        )?;
        business_case.lines = statement
            .query_map(params![id], |row| {
                Ok(BusinessCaseLine {
                    id: row.get(0)?,
                    source_type: row.get(1)?,
                    product_id: row.get(2)?,
                    sku: row.get(3)?,
                    name_zh: row.get(4)?,
                    name_en: row.get(5)?,
                    quantity: row.get(6)?,
                    unit: row.get(7)?,
                    unit_price_minor: row.get(8)?,
                    amount_minor: row.get(9)?,
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
                || !matches!(
                    line.source_type.as_str(),
                    "product" | "configurable_product"
                )
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
            let product = if line.source_type == "configurable_product" {
                transaction.query_row(
                    "SELECT code, name, name, '套' FROM configurable_products
                     WHERE id = ?1 AND active = 1 AND currency = ?2",
                    params![line.product_id, input.currency.trim().to_uppercase()],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    },
                )?
            } else {
                transaction.query_row(
                    "SELECT sku, name_zh, name_en, unit FROM products
                     WHERE id = ?1 AND active = 1 AND record_type = 'standard'",
                    params![line.product_id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    },
                )?
            };
            if let Some(existing_line_id) = &line.id {
                let existing_source = transaction
                    .query_row(
                        "SELECT source_type, product_id FROM trade_case_lines
                         WHERE id = ?1 AND trade_case_id = ?2",
                        params![existing_line_id, id],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )
                    .optional()?;
                if let Some((existing_source_type, existing_product_id)) = existing_source {
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
                        && (existing_source_type != line.source_type
                            || existing_product_id != line.product_id
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
                    id, trade_case_id, sort_order, source_type, product_id, sku_snapshot,
                    name_zh_snapshot, name_en_snapshot, quantity, unit_snapshot,
                    unit_price_minor, amount_minor
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                    sort_order = excluded.sort_order,
                    source_type = excluded.source_type,
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
                    line.source_type,
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

    pub fn update_business_case_stage(
        &self,
        id: &str,
        stage: PipelineStage,
    ) -> rusqlite::Result<BusinessCase> {
        let transaction = self.connection.unchecked_transaction()?;
        let changed = transaction.execute(
            "UPDATE trade_cases SET stage = ?2 WHERE id = ?1 AND active = 1",
            params![id, stage.as_str()],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('business_case', ?1, 'update_stage', ?2)",
            params![id, format!(r#"{{"stage":"{}"}}"#, stage.as_str())],
        )?;
        transaction.commit()?;
        self.get_business_case(id)
    }

    pub fn list_cost_estimates(&self) -> rusqlite::Result<Vec<CostEstimate>> {
        let ids = {
            let mut statement = self.connection.prepare(
                "SELECT id FROM cost_estimates WHERE active = 1
                 ORDER BY updated_at DESC, number COLLATE NOCASE DESC",
            )?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        ids.iter().map(|id| self.get_cost_estimate(id)).collect()
    }

    pub fn get_cost_estimate(&self, id: &str) -> rusqlite::Result<CostEstimate> {
        let mut estimate = self.connection.query_row(
            "SELECT id, number, trade_case_id, trade_case_number_snapshot,
                    customer_name_snapshot, currency, target_margin_bps, notes,
                    total_cost_minor, suggested_price_minor, updated_at
             FROM cost_estimates WHERE id = ?1 AND active = 1",
            params![id],
            |row| {
                Ok(CostEstimate {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    business_case_id: row.get(2)?,
                    business_case_number: row.get(3)?,
                    customer_name: row.get(4)?,
                    currency: row.get(5)?,
                    target_margin_bps: row.get(6)?,
                    notes: row.get(7)?,
                    total_cost_minor: row.get(8)?,
                    suggested_price_minor: row.get(9)?,
                    updated_at: row.get(10)?,
                    lines: Vec::new(),
                })
            },
        )?;
        let mut statement = self.connection.prepare(
            "SELECT id, category, description, specification, quantity, unit,
                    unit_cost_minor, amount_minor, notes
             FROM cost_estimate_lines WHERE cost_estimate_id = ?1 ORDER BY sort_order",
        )?;
        estimate.lines = statement
            .query_map(params![id], |row| {
                Ok(CostEstimateLine {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    description: row.get(2)?,
                    specification: row.get(3)?,
                    quantity: row.get(4)?,
                    unit: row.get(5)?,
                    unit_cost_minor: row.get(6)?,
                    amount_minor: row.get(7)?,
                    notes: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(estimate)
    }

    pub fn save_cost_estimate(&self, input: CostEstimateInput) -> rusqlite::Result<CostEstimate> {
        require_text(&input.number)?;
        require_text(&input.business_case_id)?;
        if input.target_margin_bps < 0
            || input.target_margin_bps > 9_500
            || input.lines.is_empty()
            || input.lines.iter().any(|line| {
                !COST_CATEGORIES.contains(&line.category.as_str())
                    || line.description.trim().is_empty()
                    || line.unit.trim().is_empty()
                    || !line.quantity.is_finite()
                    || line.quantity <= 0.0
                    || line.unit_cost_minor < 0
            })
        {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let transaction = self.connection.unchecked_transaction()?;
        let (case_number, customer_name, currency) = transaction.query_row(
            "SELECT number, customer_name_snapshot, currency
             FROM trade_cases WHERE id = ?1 AND active = 1",
            params![input.business_case_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let mut prepared_lines = Vec::with_capacity(input.lines.len());
        let mut total_cost_minor = 0_i64;
        for line in &input.lines {
            let amount_minor = (line.quantity * line.unit_cost_minor as f64).round() as i64;
            total_cost_minor = total_cost_minor
                .checked_add(amount_minor)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            prepared_lines.push((
                line,
                line.id
                    .clone()
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
                amount_minor,
            ));
        }
        let denominator = 10_000_i128 - input.target_margin_bps as i128;
        let suggested = ((total_cost_minor as i128 * 10_000) + denominator / 2) / denominator;
        let suggested_price_minor =
            i64::try_from(suggested).map_err(|_| rusqlite::Error::InvalidQuery)?;

        transaction.execute(
            "INSERT INTO cost_estimates(
                id, number, trade_case_id, trade_case_number_snapshot,
                customer_name_snapshot, currency, target_margin_bps, notes,
                total_cost_minor, suggested_price_minor, active, updated_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                number = excluded.number,
                trade_case_id = excluded.trade_case_id,
                trade_case_number_snapshot = excluded.trade_case_number_snapshot,
                customer_name_snapshot = excluded.customer_name_snapshot,
                currency = excluded.currency,
                target_margin_bps = excluded.target_margin_bps,
                notes = excluded.notes,
                total_cost_minor = excluded.total_cost_minor,
                suggested_price_minor = excluded.suggested_price_minor,
                active = 1,
                updated_at = CURRENT_TIMESTAMP",
            params![
                id,
                input.number.trim(),
                input.business_case_id,
                case_number,
                customer_name,
                currency,
                input.target_margin_bps,
                input.notes.trim(),
                total_cost_minor,
                suggested_price_minor,
            ],
        )?;
        transaction.execute(
            "DELETE FROM cost_estimate_lines WHERE cost_estimate_id = ?1",
            params![id],
        )?;
        for (index, (line, line_id, amount_minor)) in prepared_lines.iter().enumerate() {
            transaction.execute(
                "INSERT INTO cost_estimate_lines(
                    id, cost_estimate_id, sort_order, category, description,
                    specification, quantity, unit, unit_cost_minor, amount_minor, notes
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    line_id,
                    id,
                    index as i64,
                    line.category,
                    line.description.trim(),
                    line.specification.trim(),
                    line.quantity,
                    line.unit.trim(),
                    line.unit_cost_minor,
                    amount_minor,
                    line.notes.trim(),
                ],
            )?;
        }
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('cost_estimate', ?1, 'save', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.get_cost_estimate(&id)
    }

    pub fn archive_cost_estimate(&self, id: &str) -> rusqlite::Result<()> {
        let changed = self.connection.execute(
            "UPDATE cost_estimates SET active = 0, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND active = 1",
            params![id],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.audit("cost_estimate", id, "archive")
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

    pub fn list_partners(&self) -> rusqlite::Result<Vec<Partner>> {
        let mut statement = self.connection.prepare(
            "SELECT id, code, legal_name, partner_type, contact, address, active
             FROM partners WHERE active = 1 ORDER BY code COLLATE NOCASE",
        )?;
        statement
            .query_map([], |row| {
                Ok(Partner {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    legal_name: row.get(2)?,
                    partner_type: row.get(3)?,
                    contact: row.get(4)?,
                    address: row.get(5)?,
                    active: row.get::<_, i64>(6)? != 0,
                })
            })?
            .collect()
    }

    pub fn save_partner(&self, input: PartnerInput) -> rusqlite::Result<Partner> {
        require_text(&input.code)?;
        require_text(&input.legal_name)?;
        if !matches!(
            input.partner_type.as_str(),
            "freight_forwarder" | "customs_broker" | "insurer" | "inspection" | "other"
        ) {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.connection.execute(
            "INSERT INTO partners(id, code, legal_name, partner_type, contact, address, active)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 1)
             ON CONFLICT(id) DO UPDATE SET
                code = excluded.code, legal_name = excluded.legal_name,
                partner_type = excluded.partner_type, contact = excluded.contact,
                address = excluded.address, active = 1",
            params![
                id,
                input.code.trim(),
                input.legal_name.trim(),
                input.partner_type,
                input.contact.trim(),
                input.address.trim(),
            ],
        )?;
        self.audit("partner", &id, "save")?;
        self.connection.query_row(
            "SELECT id, code, legal_name, partner_type, contact, address, active
             FROM partners WHERE id = ?1",
            params![id],
            |row| {
                Ok(Partner {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    legal_name: row.get(2)?,
                    partner_type: row.get(3)?,
                    contact: row.get(4)?,
                    address: row.get(5)?,
                    active: row.get::<_, i64>(6)? != 0,
                })
            },
        )
    }

    pub fn archive_partner(&self, id: &str) -> rusqlite::Result<()> {
        let changed = self
            .connection
            .execute("UPDATE partners SET active = 0 WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.audit("partner", id, "archive")
    }

    pub fn list_shipment_batches(&self) -> rusqlite::Result<Vec<ShipmentBatch>> {
        let ids = {
            let mut statement = self.connection.prepare(
                "SELECT id FROM shipment_batches WHERE active = 1
                 ORDER BY planned_date DESC, number COLLATE NOCASE DESC",
            )?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        ids.iter().map(|id| self.get_shipment_batch(id)).collect()
    }

    pub fn get_shipment_batch(&self, id: &str) -> rusqlite::Result<ShipmentBatch> {
        let mut batch = self.connection.query_row(
            "SELECT id, number, trade_case_id, trade_case_number_snapshot,
                    COALESCE(partner_id, ''), partner_name_snapshot, status, planned_date,
                    actual_date, tracking_number, notes
             FROM shipment_batches WHERE id = ?1 AND active = 1",
            params![id],
            |row| {
                let status_value: String = row.get(6)?;
                Ok(ShipmentBatch {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    business_case_id: row.get(2)?,
                    business_case_number: row.get(3)?,
                    partner_id: row.get(4)?,
                    partner_name: row.get(5)?,
                    status: ShipmentStatus::from_db(&status_value)
                        .ok_or(rusqlite::Error::InvalidQuery)?,
                    planned_date: row.get(7)?,
                    actual_date: row.get(8)?,
                    tracking_number: row.get(9)?,
                    notes: row.get(10)?,
                    lines: Vec::new(),
                })
            },
        )?;
        let mut statement = self.connection.prepare(
            "SELECT id, trade_case_line_id, sku_snapshot, product_name_snapshot, quantity, unit_snapshot
             FROM shipment_batch_lines WHERE shipment_batch_id = ?1 ORDER BY sort_order",
        )?;
        batch.lines = statement
            .query_map(params![id], |row| {
                Ok(ShipmentLine {
                    id: row.get(0)?,
                    business_case_line_id: row.get(1)?,
                    sku: row.get(2)?,
                    product_name: row.get(3)?,
                    quantity: row.get(4)?,
                    unit: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(batch)
    }

    pub fn save_shipment_batch(
        &self,
        input: ShipmentBatchInput,
    ) -> rusqlite::Result<ShipmentBatch> {
        require_text(&input.number)?;
        require_text(&input.business_case_id)?;
        if input.lines.is_empty()
            || input.lines.iter().any(|line| {
                line.business_case_line_id.trim().is_empty()
                    || !line.quantity.is_finite()
                    || line.quantity <= 0.0
            })
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let mut unique_lines = std::collections::HashSet::new();
        if input
            .lines
            .iter()
            .any(|line| !unique_lines.insert(line.business_case_line_id.as_str()))
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let transaction = self.connection.unchecked_transaction()?;
        let case_number = transaction.query_row(
            "SELECT number FROM trade_cases WHERE id = ?1 AND active = 1",
            params![input.business_case_id],
            |row| row.get::<_, String>(0),
        )?;
        let (partner_id, partner_name) = if input.partner_id.trim().is_empty() {
            (None, String::new())
        } else {
            let name = transaction.query_row(
                "SELECT legal_name FROM partners WHERE id = ?1 AND active = 1",
                params![input.partner_id],
                |row| row.get::<_, String>(0),
            )?;
            (Some(input.partner_id.clone()), name)
        };
        let mut prepared = Vec::with_capacity(input.lines.len());
        for line in &input.lines {
            let snapshot = transaction.query_row(
                "SELECT sku_snapshot, COALESCE(NULLIF(name_en_snapshot, ''), name_zh_snapshot), quantity, unit_snapshot
                 FROM trade_case_lines WHERE id = ?1 AND trade_case_id = ?2",
                params![line.business_case_line_id, input.business_case_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?, row.get::<_, String>(3)?)),
            )?;
            let allocated = transaction.query_row(
                "SELECT COALESCE(SUM(sbl.quantity), 0) FROM shipment_batch_lines sbl
                 JOIN shipment_batches sb ON sb.id = sbl.shipment_batch_id
                 WHERE sbl.trade_case_line_id = ?1 AND sb.id <> ?2
                   AND sb.active = 1 AND sb.status <> 'cancelled'",
                params![line.business_case_line_id, id],
                |row| row.get::<_, f64>(0),
            )?;
            if input.status != ShipmentStatus::Cancelled
                && allocated + line.quantity > snapshot.2 + 0.000_001
            {
                return Err(rusqlite::Error::InvalidQuery);
            }
            prepared.push((line, snapshot));
        }
        transaction.execute(
            "INSERT INTO shipment_batches(
                id, number, trade_case_id, trade_case_number_snapshot, partner_id,
                partner_name_snapshot, status, planned_date, actual_date, tracking_number, notes, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)
             ON CONFLICT(id) DO UPDATE SET
                number = excluded.number, trade_case_id = excluded.trade_case_id,
                trade_case_number_snapshot = excluded.trade_case_number_snapshot,
                partner_id = excluded.partner_id, partner_name_snapshot = excluded.partner_name_snapshot,
                status = excluded.status, planned_date = excluded.planned_date,
                actual_date = excluded.actual_date, tracking_number = excluded.tracking_number,
                notes = excluded.notes, active = 1",
            params![id, input.number.trim(), input.business_case_id, case_number, partner_id,
                partner_name, input.status.as_str(), input.planned_date.trim(), input.actual_date.trim(),
                input.tracking_number.trim(), input.notes.trim()],
        )?;
        transaction.execute(
            "DELETE FROM shipment_batch_lines WHERE shipment_batch_id = ?1",
            params![id],
        )?;
        for (index, (line, snapshot)) in prepared.iter().enumerate() {
            transaction.execute(
                "INSERT INTO shipment_batch_lines(
                    id, shipment_batch_id, trade_case_line_id, sort_order, sku_snapshot,
                    product_name_snapshot, quantity, unit_snapshot
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    line.business_case_line_id,
                    index as i64,
                    snapshot.0,
                    snapshot.1,
                    line.quantity,
                    snapshot.3
                ],
            )?;
        }
        if matches!(
            input.status,
            ShipmentStatus::Shipped | ShipmentStatus::Delivered
        ) {
            transaction.execute(
                "UPDATE trade_cases SET stage = 'shipment' WHERE id = ?1 AND stage <> 'documents'",
                params![input.business_case_id],
            )?;
        }
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('shipment_batch', ?1, 'save', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.get_shipment_batch(&id)
    }

    pub fn list_payment_plans(&self) -> rusqlite::Result<Vec<PaymentPlan>> {
        let mut statement = self.connection.prepare(
            "SELECT id, number, trade_case_id, trade_case_number_snapshot, payment_type,
                    due_date, currency, amount_minor, received_amount_minor, received_date,
                    status, notes FROM payment_plans WHERE active = 1
             ORDER BY due_date DESC, number COLLATE NOCASE DESC",
        )?;
        statement.query_map([], map_payment_plan)?.collect()
    }

    pub fn save_payment_plan(&self, input: PaymentPlanInput) -> rusqlite::Result<PaymentPlan> {
        require_text(&input.number)?;
        require_text(&input.business_case_id)?;
        if !matches!(
            input.payment_type.as_str(),
            "deposit" | "balance" | "installment" | "other"
        ) || input.amount_minor <= 0
            || input.received_amount_minor < 0
            || input.received_amount_minor > input.amount_minor
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let transaction = self.connection.unchecked_transaction()?;
        let (case_number, currency, sales_amount) = transaction.query_row(
            "SELECT number, currency, sales_amount_minor FROM trade_cases WHERE id = ?1 AND active = 1",
            params![input.business_case_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
        )?;
        let planned_elsewhere = transaction.query_row(
            "SELECT COALESCE(SUM(amount_minor), 0) FROM payment_plans
             WHERE trade_case_id = ?1 AND id <> ?2 AND active = 1 AND status <> 'cancelled'",
            params![input.business_case_id, id],
            |row| row.get::<_, i64>(0),
        )?;
        if input.status != PaymentStatus::Cancelled
            && planned_elsewhere.saturating_add(input.amount_minor) > sales_amount
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let status = if input.status == PaymentStatus::Cancelled {
            PaymentStatus::Cancelled
        } else if input.received_amount_minor == 0 {
            PaymentStatus::Planned
        } else if input.received_amount_minor < input.amount_minor {
            PaymentStatus::Partial
        } else {
            PaymentStatus::Received
        };
        transaction.execute(
            "INSERT INTO payment_plans(
                id, number, trade_case_id, trade_case_number_snapshot, payment_type, due_date,
                currency, amount_minor, received_amount_minor, received_date, status, notes, active
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1)
             ON CONFLICT(id) DO UPDATE SET
                number = excluded.number, trade_case_id = excluded.trade_case_id,
                trade_case_number_snapshot = excluded.trade_case_number_snapshot,
                payment_type = excluded.payment_type, due_date = excluded.due_date,
                currency = excluded.currency, amount_minor = excluded.amount_minor,
                received_amount_minor = excluded.received_amount_minor,
                received_date = excluded.received_date, status = excluded.status,
                notes = excluded.notes, active = 1",
            params![
                id,
                input.number.trim(),
                input.business_case_id,
                case_number,
                input.payment_type,
                input.due_date.trim(),
                currency,
                input.amount_minor,
                input.received_amount_minor,
                input.received_date.trim(),
                status.as_str(),
                input.notes.trim()
            ],
        )?;
        transaction.execute(
            "INSERT INTO audit_events(entity_type, entity_id, action, payload_json)
             VALUES('payment_plan', ?1, 'save', '{}')",
            params![id],
        )?;
        transaction.commit()?;
        self.connection.query_row(
            "SELECT id, number, trade_case_id, trade_case_number_snapshot, payment_type,
                    due_date, currency, amount_minor, received_amount_minor, received_date,
                    status, notes FROM payment_plans WHERE id = ?1",
            params![id],
            map_payment_plan,
        )
    }

    pub fn list_documents(&self) -> rusqlite::Result<Vec<TradeDocument>> {
        let mut documents = self.list_documents_raw()?;
        let peers = documents.clone();
        for document in &mut documents {
            let issues = crate::document::cross_validate(document, &peers);
            document.validation_issues.extend(issues);
        }
        Ok(documents)
    }

    fn list_documents_raw(&self) -> rusqlite::Result<Vec<TradeDocument>> {
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
        let mut document = self.get_document_raw(id)?;
        let peers = self.list_documents_raw()?;
        let issues = crate::document::cross_validate(&document, &peers);
        document.validation_issues.extend(issues);
        Ok(document)
    }

    fn get_document_raw(&self, id: &str) -> rusqlite::Result<TradeDocument> {
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
            DocumentType::PackingList
            | DocumentType::ShippingMarks
            | DocumentType::ShipperInstruction
            | DocumentType::CustomsDeclaration
            | DocumentType::BillOfLading
            | DocumentType::InsurancePolicy
            | DocumentType::CertificateOfOrigin
            | DocumentType::InspectionCertificate
            | DocumentType::FumigationCertificate
            | DocumentType::BeneficiaryCertificate => &shipping_address,
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
        let sales_total_minor = lines.iter().map(|line| line.amount_minor).sum::<i64>();
        let normalized_incoterm = business_case.incoterm.trim().to_ascii_uppercase();
        let insurance_markup_percent =
            if normalized_incoterm.starts_with("CIF") || normalized_incoterm.starts_with("CIP") {
                10.0
            } else {
                0.0
            };
        let insured_value_minor =
            (sales_total_minor as f64 * (1.0 + insurance_markup_percent / 100.0)).round() as i64;
        let payload = DocumentPayload {
            seller: company_name.clone(),
            seller_address: String::new(),
            buyer: business_case.customer_name.clone(),
            buyer_address: buyer_address.clone(),
            origin_country: "China".to_owned(),
            destination_country: destination_country.clone(),
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
            shipping_marks: format!(
                "{} / {} / MADE IN CHINA",
                business_case.customer_name, business_case.number
            ),
            transport_mode: "Sea".to_owned(),
            vessel_voyage: String::new(),
            booking_reference: String::new(),
            freight_terms: "Freight Prepaid".to_owned(),
            bill_of_lading_type: "Original B/L".to_owned(),
            customs_supervision_code: String::new(),
            customs_declaration_elements: String::new(),
            notify_party: business_case.customer_name.clone(),
            notify_party_address: buyer_address.clone(),
            carrier: String::new(),
            bill_of_lading_number: String::new(),
            place_of_receipt: String::new(),
            place_of_delivery: String::new(),
            container_numbers: String::new(),
            seal_numbers: String::new(),
            insurance_company: String::new(),
            policy_number: String::new(),
            insured_value_minor,
            insurance_markup_percent,
            premium_rate_percent: 0.0,
            premium_minor: 0,
            insurance_coverage: "Institute Cargo Clauses (A)".to_owned(),
            claims_payable_at: destination_country.clone(),
            certificate_number: String::new(),
            certificate_type: "General Certificate of Origin".to_owned(),
            certification_authority: String::new(),
            manufacturer: company_name,
            manufacturer_address: String::new(),
            batch_number: String::new(),
            inspection_standard: String::new(),
            inspection_date: input.issue_date.trim().to_owned(),
            inspection_place: String::new(),
            inspection_result: "Conforms to the stated inspection standard.".to_owned(),
            fumigation_agent: "Methyl Bromide".to_owned(),
            fumigation_method: "Fumigation under gas-proof sheet".to_owned(),
            fumigation_temperature_celsius: 21.0,
            fumigation_duration_hours: 24.0,
            fumigation_date: input.issue_date.trim().to_owned(),
            fumigation_place: String::new(),
            fumigation_operator: String::new(),
            fumigation_license_number: String::new(),
            letter_of_credit_number: String::new(),
            issuing_bank: String::new(),
            letter_of_credit_issue_date: String::new(),
            letter_of_credit_expiry_date: String::new(),
            presentation_deadline: String::new(),
            beneficiary_certificate_type: "Beneficiary's Certificate".to_owned(),
            beneficiary_statement: "We hereby certify that the documents required under the letter of credit have been presented in accordance with its terms.".to_owned(),
            letter_of_credit_terms: String::new(),
            required_documents: String::new(),
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
                ) | (
                    DocumentType::CommercialInvoice,
                    DocumentType::PackingList
                        | DocumentType::ShippingMarks
                        | DocumentType::ShipperInstruction
                        | DocumentType::CustomsDeclaration
                        | DocumentType::BillOfLading
                        | DocumentType::InsurancePolicy
                        | DocumentType::CertificateOfOrigin
                        | DocumentType::InspectionCertificate
                        | DocumentType::FumigationCertificate
                        | DocumentType::BeneficiaryCertificate
                ) | (
                    DocumentType::PackingList,
                    DocumentType::ShippingMarks
                        | DocumentType::ShipperInstruction
                        | DocumentType::CustomsDeclaration
                        | DocumentType::BillOfLading
                        | DocumentType::InsurancePolicy
                        | DocumentType::CertificateOfOrigin
                        | DocumentType::InspectionCertificate
                        | DocumentType::FumigationCertificate
                        | DocumentType::BeneficiaryCertificate
                ) | (
                    DocumentType::ShipperInstruction,
                    DocumentType::BillOfLading
                        | DocumentType::InsurancePolicy
                        | DocumentType::CertificateOfOrigin
                        | DocumentType::InspectionCertificate
                        | DocumentType::FumigationCertificate
                        | DocumentType::BeneficiaryCertificate
                ) | (
                    DocumentType::BillOfLading,
                    DocumentType::InsurancePolicy
                        | DocumentType::CertificateOfOrigin
                        | DocumentType::InspectionCertificate
                        | DocumentType::FumigationCertificate
                        | DocumentType::BeneficiaryCertificate
                ) | (
                    DocumentType::CustomsDeclaration,
                    DocumentType::CertificateOfOrigin
                        | DocumentType::InspectionCertificate
                        | DocumentType::FumigationCertificate
                        | DocumentType::BeneficiaryCertificate
                )
            )
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = Uuid::new_v4().to_string();
        let mut payload = source.payload.clone();
        if input.target_document_type == DocumentType::BillOfLading {
            if payload.notify_party.trim().is_empty() {
                payload.notify_party.clone_from(&payload.buyer);
            }
            if payload.notify_party_address.trim().is_empty() {
                payload
                    .notify_party_address
                    .clone_from(&payload.buyer_address);
            }
            if payload.place_of_receipt.trim().is_empty() {
                payload
                    .place_of_receipt
                    .clone_from(&payload.port_of_loading);
            }
            if payload.place_of_delivery.trim().is_empty() {
                payload
                    .place_of_delivery
                    .clone_from(&payload.port_of_discharge);
            }
        }
        if input.target_document_type == DocumentType::InsurancePolicy {
            let incoterm = payload.incoterm.trim().to_ascii_uppercase();
            if payload.insurance_markup_percent <= 0.0
                && (incoterm.starts_with("CIF") || incoterm.starts_with("CIP"))
            {
                payload.insurance_markup_percent = 10.0;
            }
            if payload.insured_value_minor <= 0 {
                let cargo_value = payload
                    .lines
                    .iter()
                    .map(|line| line.amount_minor)
                    .sum::<i64>()
                    - payload.discount_minor;
                payload.insured_value_minor = (cargo_value as f64
                    * (1.0 + payload.insurance_markup_percent / 100.0))
                    .round() as i64;
            }
            if payload.insurance_coverage.trim().is_empty() {
                payload.insurance_coverage = "Institute Cargo Clauses (A)".to_owned();
            }
            if payload.claims_payable_at.trim().is_empty() {
                payload
                    .claims_payable_at
                    .clone_from(&payload.destination_country);
            }
        }
        if input.target_document_type == DocumentType::CertificateOfOrigin
            && payload.certificate_type.trim().is_empty()
        {
            payload.certificate_type = "General Certificate of Origin".to_owned();
        }
        if input.target_document_type == DocumentType::InspectionCertificate {
            if payload.manufacturer.trim().is_empty() {
                payload.manufacturer.clone_from(&payload.seller);
            }
            payload.inspection_date = input.issue_date.trim().to_owned();
            if payload.inspection_result.trim().is_empty() {
                payload.inspection_result =
                    "Conforms to the stated inspection standard.".to_owned();
            }
        }
        if input.target_document_type == DocumentType::FumigationCertificate {
            payload.fumigation_date = input.issue_date.trim().to_owned();
            if payload.fumigation_agent.trim().is_empty() {
                payload.fumigation_agent = "Methyl Bromide".to_owned();
            }
            if payload.fumigation_method.trim().is_empty() {
                payload.fumigation_method = "Fumigation under gas-proof sheet".to_owned();
            }
            if payload.fumigation_duration_hours <= 0.0 {
                payload.fumigation_duration_hours = 24.0;
            }
        }
        if input.target_document_type == DocumentType::BeneficiaryCertificate
            && payload.beneficiary_certificate_type.trim().is_empty()
        {
            payload.beneficiary_certificate_type = "Beneficiary's Certificate".to_owned();
        }
        let payload_json = serde_json::to_string(&payload).map_err(json_error)?;
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
        if payload.insured_value_minor < 0
            || payload.premium_minor < 0
            || !payload.insurance_markup_percent.is_finite()
            || payload.insurance_markup_percent < 0.0
            || payload.insurance_markup_percent > 1000.0
            || !payload.premium_rate_percent.is_finite()
            || payload.premium_rate_percent < 0.0
            || payload.premium_rate_percent > 100.0
            || !payload.fumigation_temperature_celsius.is_finite()
            || payload.fumigation_temperature_celsius < -100.0
            || payload.fumigation_temperature_celsius > 200.0
            || !payload.fumigation_duration_hours.is_finite()
            || payload.fumigation_duration_hours < 0.0
            || payload.fumigation_duration_hours > 10_000.0
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        payload.premium_minor = (payload.insured_value_minor as f64 * payload.premium_rate_percent
            / 100.0)
            .round() as i64;
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
            DocumentType::CommercialInvoice
            | DocumentType::PackingList
            | DocumentType::ShippingMarks
            | DocumentType::ShipperInstruction
            | DocumentType::CustomsDeclaration
            | DocumentType::BillOfLading
            | DocumentType::InsurancePolicy
            | DocumentType::CertificateOfOrigin
            | DocumentType::InspectionCertificate
            | DocumentType::FumigationCertificate
            | DocumentType::BeneficiaryCertificate => {
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

    pub fn backup_to(&self, path: &Path) -> rusqlite::Result<()> {
        let mut destination = Connection::open(path)?;
        destination.pragma_update(None, "key", self.key.as_str())?;
        destination.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;",
        )?;
        let backup = rusqlite::backup::Backup::new(&self.connection, &mut destination)?;
        backup.run_to_completion(128, Duration::from_millis(10), None)?;
        drop(backup);
        destination
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .and_then(|result| {
                if result == "ok" {
                    Ok(())
                } else {
                    Err(rusqlite::Error::InvalidQuery)
                }
            })
    }

    pub fn save_attachment(&self, input: AttachmentInput) -> rusqlite::Result<AttachmentRecord> {
        let entity_type = input.entity_type.trim();
        let file_name = input.file_name.trim();
        if entity_type.is_empty()
            || file_name.is_empty()
            || input.bytes.is_empty()
            || input.bytes.len() > 20 * 1024 * 1024
        {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let id = Uuid::new_v4().to_string();
        let size_bytes = input.bytes.len() as i64;
        let sha256 = format!("{:x}", Sha256::digest(&input.bytes));
        self.connection.execute(
            "INSERT INTO attachments(
                id, entity_type, entity_id, entity_label, file_name, mime_type, content, size_bytes, sha256
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                entity_type,
                input.entity_id.trim(),
                input.entity_label.trim(),
                file_name,
                if input.mime_type.trim().is_empty() {
                    "application/octet-stream"
                } else {
                    input.mime_type.trim()
                },
                input.bytes,
                size_bytes,
                sha256,
            ],
        )?;
        self.audit("attachment", &id, "create")?;
        self.get_attachment(&id)
    }

    pub fn list_attachments(&self) -> rusqlite::Result<Vec<AttachmentRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, entity_type, entity_id, entity_label, file_name, mime_type, size_bytes, sha256, created_at
             FROM attachments ORDER BY created_at DESC, id DESC",
        )?;
        statement.query_map([], map_attachment)?.collect()
    }

    pub fn list_entity_attachments(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> rusqlite::Result<Vec<AttachmentRecord>> {
        require_text(entity_type)?;
        require_text(entity_id)?;
        let mut statement = self.connection.prepare(
            "SELECT id, entity_type, entity_id, entity_label, file_name, mime_type, size_bytes, sha256, created_at
             FROM attachments WHERE entity_type = ?1 AND entity_id = ?2
             ORDER BY created_at DESC, id DESC",
        )?;
        statement
            .query_map(
                params![entity_type.trim(), entity_id.trim()],
                map_attachment,
            )?
            .collect()
    }

    pub fn get_attachment(&self, id: &str) -> rusqlite::Result<AttachmentRecord> {
        self.connection.query_row(
            "SELECT id, entity_type, entity_id, entity_label, file_name, mime_type, size_bytes, sha256, created_at
             FROM attachments WHERE id = ?1",
            params![id],
            map_attachment,
        )
    }

    pub fn attachment_content(&self, id: &str) -> rusqlite::Result<(String, Vec<u8>)> {
        self.connection.query_row(
            "SELECT file_name, content FROM attachments WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    }

    pub fn delete_attachment(&self, id: &str) -> rusqlite::Result<()> {
        let changed = self
            .connection
            .execute("DELETE FROM attachments WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.audit("attachment", id, "delete")
    }

    pub fn save_document_draft(&self, input: SaveDocumentInput) -> rusqlite::Result<DocumentDraft> {
        let document = self.get_document(&input.id)?;
        if document.status != DocumentStatus::Draft {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let draft_key = format!("document:{}", input.id);
        let payload_json = serde_json::to_string(&input).map_err(json_error)?;
        self.connection.execute(
            "INSERT INTO drafts(draft_key, payload_json, updated_at)
             VALUES(?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(draft_key) DO UPDATE SET
                payload_json = excluded.payload_json,
                updated_at = CURRENT_TIMESTAMP",
            params![draft_key, payload_json],
        )?;
        self.load_document_draft(&input.id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn load_document_draft(
        &self,
        document_id: &str,
    ) -> rusqlite::Result<Option<DocumentDraft>> {
        let draft_key = format!("document:{document_id}");
        self.connection
            .query_row(
                "SELECT payload_json, updated_at FROM drafts WHERE draft_key = ?1",
                params![draft_key],
                |row| {
                    let payload_json: String = row.get(0)?;
                    Ok(DocumentDraft {
                        input: serde_json::from_str(&payload_json).map_err(json_error)?,
                        updated_at: row.get(1)?,
                    })
                },
            )
            .optional()
    }

    pub fn delete_document_draft(&self, document_id: &str) -> rusqlite::Result<()> {
        let draft_key = format!("document:{document_id}");
        self.connection.execute(
            "DELETE FROM drafts WHERE draft_key = ?1",
            params![draft_key],
        )?;
        Ok(())
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
            "products" => {
                "SELECT COUNT(*) FROM products WHERE active = 1 AND record_type = 'standard'"
            }
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

fn map_attachment(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttachmentRecord> {
    Ok(AttachmentRecord {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        entity_id: row.get(2)?,
        entity_label: row.get(3)?,
        file_name: row.get(4)?,
        mime_type: row.get(5)?,
        size_bytes: row.get::<_, i64>(6)? as u64,
        sha256: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn map_payment_plan(row: &rusqlite::Row<'_>) -> rusqlite::Result<PaymentPlan> {
    let status_value: String = row.get(10)?;
    Ok(PaymentPlan {
        id: row.get(0)?,
        number: row.get(1)?,
        business_case_id: row.get(2)?,
        business_case_number: row.get(3)?,
        payment_type: row.get(4)?,
        due_date: row.get(5)?,
        currency: row.get(6)?,
        amount_minor: row.get(7)?,
        received_amount_minor: row.get(8)?,
        received_date: row.get(9)?,
        status: PaymentStatus::from_db(&status_value).ok_or(rusqlite::Error::InvalidQuery)?,
        notes: row.get(11)?,
    })
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

fn validate_component_option_kind(value: &str) -> rusqlite::Result<()> {
    if matches!(
        value,
        "category"
            | "name"
            | "brand"
            | "specification"
            | "unit"
            | "notes"
            | "product_name"
            | "configuration_notes"
    ) {
        Ok(())
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

fn validate_configuration_language(value: &str) -> rusqlite::Result<()> {
    if matches!(value, "en" | "ru" | "fr" | "es" | "pt" | "ar") {
        Ok(())
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

fn contains_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}'))
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
    use crate::domain::{BusinessCaseLineInput, ConfigurableProductLineInput, ShipmentLineInput};

    #[test]
    fn encrypted_attachments_and_online_backup_round_trip() {
        let root = std::env::temp_dir().join(format!("tradedesk-attachment-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("workspace.tdesk");
        let backup_path = root.join("backup.tdesk");
        let secret_bytes = b"confidential purchase confirmation".to_vec();
        let attachment_id;
        {
            let database =
                EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
            let saved = database
                .save_attachment(AttachmentInput {
                    entity_type: "purchase_order".into(),
                    entity_id: "PO-2026-0001".into(),
                    entity_label: "PO-2026-0001".into(),
                    file_name: "confirmation.txt".into(),
                    mime_type: "text/plain".into(),
                    bytes: secret_bytes.clone(),
                })
                .unwrap();
            attachment_id = saved.id;
            assert_eq!(saved.size_bytes, secret_bytes.len() as u64);
            assert_eq!(
                database.attachment_content(&attachment_id).unwrap().1,
                secret_bytes
            );
            database.backup_to(&backup_path).unwrap();
        }
        let raw_database = std::fs::read(&path).unwrap();
        assert!(
            !raw_database
                .windows(secret_bytes.len())
                .any(|window| window == secret_bytes)
        );
        let backup =
            EncryptedDatabase::open(&backup_path, Zeroizing::new("test-password".to_owned()))
                .unwrap();
        assert_eq!(
            backup.attachment_content(&attachment_id).unwrap().1,
            secret_bytes
        );
        drop(backup);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn encrypted_database_persists_business_workflow() {
        let path = std::env::temp_dir().join(format!("tradedesk-{}.db", Uuid::new_v4()));
        {
            let database =
                EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
            let company_registry = database
                .save_company_registry(CompanyRegistry {
                    default_company_id: "company-test".into(),
                    companies: vec![CompanyRecord {
                        id: "company-test".into(),
                        company_name: "Example Export Co., Ltd.".into(),
                        logo_data_url: String::new(),
                        signing_assets: vec![CompanySigningAsset {
                            id: "stamp-test".into(),
                            name: "QA Stamp".into(),
                            kind: "stamp".into(),
                            data_url: format!(
                                "data:image/png;base64,{}",
                                base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    include_bytes!("../icons/32x32.png")
                                )
                            ),
                        }],
                    }],
                })
                .unwrap();
            assert_eq!(
                company_registry.companies[0].company_name,
                "Example Export Co., Ltd."
            );
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
                    exchange_rate: 1.0,
                    exchange_rate_date: String::new(),
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
            let options = database.list_component_options().unwrap();
            assert_eq!(
                options
                    .iter()
                    .filter(|option| option.kind == "category")
                    .count(),
                2
            );
            assert!(
                options
                    .iter()
                    .any(|option| option.kind == "brand" && option.value == "康达")
            );
            let configuration_customer = database
                .save_customer(CustomerInput {
                    id: None,
                    code: "CUS-CFG".into(),
                    legal_name: "Configuration Buyer".into(),
                    market: "CN".into(),
                    currency: "CNY".into(),
                    payment_terms: "T/T".into(),
                    address: String::new(),
                    shipping_address: String::new(),
                    billing_address: String::new(),
                    purchase_intent: String::new(),
                    customer_analysis: String::new(),
                    strengths: String::new(),
                    weaknesses: String::new(),
                    contacts: String::new(),
                })
                .unwrap();
            let configuration_case = database
                .save_business_case(BusinessCaseInput {
                    id: None,
                    number: "TD-2026-CFG".into(),
                    customer_id: configuration_customer.id,
                    stage: PipelineStage::Quotation,
                    currency: "CNY".into(),
                    incoterm: "EXW".into(),
                    payment_terms: "T/T".into(),
                    shipment_date: String::new(),
                    notes: String::new(),
                    lines: vec![BusinessCaseLineInput {
                        id: None,
                        source_type: "configurable_product".into(),
                        product_id: configured.id.clone(),
                        quantity: 1.0,
                        unit_price_minor: configured.total_amount_minor,
                    }],
                })
                .unwrap();
            assert_eq!(
                configuration_case.lines[0].source_type,
                "configurable_product"
            );
            assert_eq!(configuration_case.lines[0].sku, "CFG-K38-G6");
            database
                .archive_business_case(&configuration_case.id)
                .unwrap();
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
                        source_type: "product".into(),
                        product_id: product.id,
                        quantity: 12.5,
                        unit_price_minor: 240,
                    }],
                })
                .unwrap();
            assert_eq!(business_case.total_amount_minor, 3_000);
            let updated_stage = database
                .update_business_case_stage(&business_case.id, PipelineStage::Production)
                .unwrap();
            assert_eq!(updated_stage.stage, PipelineStage::Production);
            let business_case = database
                .update_business_case_stage(&business_case.id, PipelineStage::Quotation)
                .unwrap();
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
            let cost_estimate = database
                .save_cost_estimate(crate::domain::CostEstimateInput {
                    id: None,
                    number: "CST-20260811-0001".into(),
                    business_case_id: business_case.id.clone(),
                    target_margin_bps: 3_000,
                    notes: "complete quotation cost".into(),
                    lines: vec![
                        crate::domain::CostEstimateLineInput {
                            id: None,
                            category: "material".into(),
                            description: "Purchased product".into(),
                            specification: "SUP-1".into(),
                            quantity: 12.5,
                            unit: "set".into(),
                            unit_cost_minor: 160,
                            notes: "PO-2026-0001".into(),
                        },
                        crate::domain::CostEstimateLineInput {
                            id: None,
                            category: "international_freight".into(),
                            description: "Ocean freight".into(),
                            specification: "FOB surcharge".into(),
                            quantity: 1.0,
                            unit: "lot".into(),
                            unit_cost_minor: 100,
                            notes: String::new(),
                        },
                    ],
                })
                .unwrap();
            assert_eq!(cost_estimate.currency, "USD");
            assert_eq!(cost_estimate.total_cost_minor, 2_100);
            assert_eq!(cost_estimate.suggested_price_minor, 3_000);
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
                            source_type: "product".into(),
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
            let freight_partner = database
                .save_partner(PartnerInput {
                    id: None,
                    code: "FWD-001".into(),
                    legal_name: "Example Freight Ltd.".into(),
                    partner_type: "freight_forwarder".into(),
                    contact: "ops@example.test".into(),
                    address: "Shanghai".into(),
                })
                .unwrap();
            let shipment = database
                .save_shipment_batch(ShipmentBatchInput {
                    id: None,
                    number: "SHP-20260810-0001".into(),
                    business_case_id: business_case.id.clone(),
                    partner_id: freight_partner.id,
                    status: ShipmentStatus::Booked,
                    planned_date: "2026-09-18".into(),
                    actual_date: String::new(),
                    tracking_number: "BOOKING-001".into(),
                    notes: String::new(),
                    lines: vec![ShipmentLineInput {
                        business_case_line_id: business_case.lines[0].id.clone(),
                        quantity: 7.5,
                    }],
                })
                .unwrap();
            assert_eq!(shipment.lines[0].quantity, 7.5);
            assert!(
                database
                    .save_shipment_batch(ShipmentBatchInput {
                        id: None,
                        number: "SHP-20260810-0002".into(),
                        business_case_id: business_case.id.clone(),
                        partner_id: String::new(),
                        status: ShipmentStatus::Planned,
                        planned_date: "2026-09-19".into(),
                        actual_date: String::new(),
                        tracking_number: String::new(),
                        notes: String::new(),
                        lines: vec![ShipmentLineInput {
                            business_case_line_id: business_case.lines[0].id.clone(),
                            quantity: 6.0,
                        }],
                    })
                    .is_err()
            );
            let payment = database
                .save_payment_plan(PaymentPlanInput {
                    id: None,
                    number: "PAY-20260810-0001".into(),
                    business_case_id: business_case.id.clone(),
                    payment_type: "deposit".into(),
                    due_date: "2026-08-20".into(),
                    amount_minor: 1_000,
                    received_amount_minor: 500,
                    received_date: "2026-08-18".into(),
                    status: PaymentStatus::Planned,
                    notes: String::new(),
                })
                .unwrap();
            assert_eq!(payment.status, PaymentStatus::Partial);
            assert!(
                database
                    .save_payment_plan(PaymentPlanInput {
                        id: None,
                        number: "PAY-20260810-0002".into(),
                        business_case_id: business_case.id.clone(),
                        payment_type: "balance".into(),
                        due_date: "2026-09-10".into(),
                        amount_minor: 2_001,
                        received_amount_minor: 0,
                        received_date: String::new(),
                        status: PaymentStatus::Planned,
                        notes: String::new(),
                    })
                    .is_err()
            );
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
            let draft_input = SaveDocumentInput {
                id: draft.id.clone(),
                number: draft.number.clone(),
                language: draft.language.clone(),
                issue_date: draft.issue_date.clone(),
                payload: payload.clone(),
            };
            database.save_document_draft(draft_input.clone()).unwrap();
            let recovered_draft = database.load_document_draft(&draft.id).unwrap().unwrap();
            assert_eq!(recovered_draft.input.payload.buyer_address, "Seattle, USA");
            database.delete_document_draft(&draft.id).unwrap();
            assert!(database.load_document_draft(&draft.id).unwrap().is_none());
            let saved = database.save_document(draft_input).unwrap();
            assert!(
                saved
                    .validation_issues
                    .iter()
                    .all(|issue| { issue.severity != crate::domain::ValidationSeverity::Error })
            );
            let issued = database.issue_document(&saved.id).unwrap();
            let bill_draft = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued.id.clone(),
                    target_document_type: DocumentType::BillOfLading,
                    number: "BL-20260811-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-11".into(),
                })
                .unwrap();
            assert_eq!(bill_draft.payload.notify_party, issued.payload.buyer);
            let insurance_draft = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued.id.clone(),
                    target_document_type: DocumentType::InsurancePolicy,
                    number: "INS-20260811-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-11".into(),
                })
                .unwrap();
            assert_eq!(insurance_draft.payload.insured_value_minor, 3_000);
            assert_eq!(
                insurance_draft.payload.insurance_coverage,
                "Institute Cargo Clauses (A)"
            );
            let origin_draft = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued.id.clone(),
                    target_document_type: DocumentType::CertificateOfOrigin,
                    number: "COO-20260811-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-11".into(),
                })
                .unwrap();
            assert_eq!(
                origin_draft.payload.certificate_type,
                "General Certificate of Origin"
            );
            let inspection_draft = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued.id.clone(),
                    target_document_type: DocumentType::InspectionCertificate,
                    number: "IC-20260811-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-11".into(),
                })
                .unwrap();
            assert_eq!(inspection_draft.payload.manufacturer, issued.payload.seller);
            assert_eq!(inspection_draft.payload.inspection_date, "2026-08-11");
            let fumigation_draft = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued.id.clone(),
                    target_document_type: DocumentType::FumigationCertificate,
                    number: "FUM-20260811-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-11".into(),
                })
                .unwrap();
            assert_eq!(fumigation_draft.payload.fumigation_date, "2026-08-11");
            assert_eq!(fumigation_draft.payload.fumigation_duration_hours, 24.0);
            let beneficiary_draft = database
                .convert_document(ConvertDocumentInput {
                    source_document_id: issued.id.clone(),
                    target_document_type: DocumentType::BeneficiaryCertificate,
                    number: "BC-20260811-0001".into(),
                    language: "zh_en".into(),
                    issue_date: "2026-08-11".into(),
                })
                .unwrap();
            assert_eq!(
                beneficiary_draft.payload.beneficiary_certificate_type,
                "Beneficiary's Certificate"
            );
            assert_eq!(issued.status, DocumentStatus::Issued);
            if let Some(typst) = crate::document::find_typst(std::path::Path::new("")) {
                let render_root =
                    std::env::temp_dir().join(format!("tradedesk-pdf-{}", Uuid::new_v4()));
                let work_dir = render_root.join("work");
                let output_dir = render_root.join("output");
                let company_profile = database.resolve_company_profile("", "stamp-test").unwrap();
                let export = crate::document::export_pdf(
                    &issued,
                    &company_profile,
                    &typst,
                    &work_dir,
                    &output_dir,
                )
                .unwrap();
                let pdf = std::fs::read(&export.path).unwrap();
                assert_eq!(&pdf[..5], b"%PDF-");
                assert_eq!(export.sha256.len(), 64);
                if let Ok(qa_dir) = std::env::var("TRADEDESK_PDF_QA_DIR") {
                    std::fs::create_dir_all(&qa_dir).unwrap();
                    std::fs::copy(
                        &export.path,
                        std::path::Path::new(&qa_dir).join("commercial_invoice.pdf"),
                    )
                    .unwrap();
                }
                if let Ok(output) = std::env::var("TRADEDESK_PDF_OUTPUT") {
                    std::fs::copy(&export.path, output).unwrap();
                }
                let csv = crate::document::export_csv(&issued, &output_dir).unwrap();
                let csv_content = std::fs::read_to_string(csv).unwrap();
                assert!(csv_content.contains("document_type,document_number,business_case"));
                assert!(csv_content.contains("SKU-1"));
                for sales_document in [&issued_quote, &issued_proforma] {
                    let sales_export = crate::document::export_pdf(
                        sales_document,
                        &company_profile,
                        &typst,
                        &work_dir,
                        &output_dir,
                    )
                    .unwrap();
                    assert_eq!(&std::fs::read(&sales_export.path).unwrap()[..5], b"%PDF-");
                    if let Ok(qa_dir) = std::env::var("TRADEDESK_PDF_QA_DIR") {
                        std::fs::create_dir_all(&qa_dir).unwrap();
                        std::fs::copy(
                            &sales_export.path,
                            std::path::Path::new(&qa_dir)
                                .join(format!("{}.pdf", sales_document.document_type.as_str())),
                        )
                        .unwrap();
                    }
                }
                for document_type in [
                    DocumentType::PackingList,
                    DocumentType::TradeContract,
                    DocumentType::ShippingMarks,
                    DocumentType::ShipperInstruction,
                    DocumentType::CustomsDeclaration,
                    DocumentType::BillOfLading,
                    DocumentType::InsurancePolicy,
                    DocumentType::CertificateOfOrigin,
                    DocumentType::InspectionCertificate,
                    DocumentType::FumigationCertificate,
                    DocumentType::BeneficiaryCertificate,
                ] {
                    let mut template_document = issued.clone();
                    template_document.document_type = document_type.clone();
                    let template_export = crate::document::export_pdf(
                        &template_document,
                        &company_profile,
                        &typst,
                        &work_dir,
                        &output_dir,
                    )
                    .unwrap();
                    assert_eq!(
                        &std::fs::read(&template_export.path).unwrap()[..5],
                        b"%PDF-"
                    );
                    if let Ok(qa_dir) = std::env::var("TRADEDESK_PDF_QA_DIR") {
                        std::fs::create_dir_all(&qa_dir).unwrap();
                        std::fs::copy(
                            &template_export.path,
                            std::path::Path::new(&qa_dir)
                                .join(format!("{}.pdf", document_type.as_str())),
                        )
                        .unwrap();
                    }
                }
                for language in ["en", "ru", "fr", "es", "pt", "ar"] {
                    let configuration_export = crate::document::export_configuration_pdf(
                        &configured,
                        language,
                        &company_profile,
                        &typst,
                        &work_dir,
                        &output_dir,
                    )
                    .unwrap();
                    assert_eq!(
                        &std::fs::read(&configuration_export.path).unwrap()[..5],
                        b"%PDF-"
                    );
                    if language == "en"
                        && let Ok(qa_dir) = std::env::var("TRADEDESK_PDF_QA_DIR")
                    {
                        std::fs::create_dir_all(&qa_dir).unwrap();
                        std::fs::copy(
                            &configuration_export.path,
                            std::path::Path::new(&qa_dir).join("configuration-sheet.pdf"),
                        )
                        .unwrap();
                    }
                }
                let configuration_csv =
                    crate::document::export_configuration_csv(&configured, "en", &output_dir)
                        .unwrap();
                assert!(
                    std::fs::read_to_string(configuration_csv)
                        .unwrap()
                        .contains("润滑油补给箱")
                );
                let _ = std::fs::remove_dir_all(render_root);
            }
            let mut customs = issued.clone();
            customs.id = "customs-validation".into();
            customs.document_type = DocumentType::CustomsDeclaration;
            customs.number = "CUS-20260810-0001".into();
            assert!(
                crate::document::cross_validate(&customs, std::slice::from_ref(&issued)).is_empty()
            );
            customs.payload.lines[0].quantity += 1.0;
            customs.payload.lines[0].unit_price_minor += 1;
            customs.payload.lines[0].amount_minor = (customs.payload.lines[0].quantity
                * customs.payload.lines[0].unit_price_minor as f64)
                .round() as i64;
            customs.payload.lines[0].hs_code = "DIFFERENT".into();
            let cross_issues =
                crate::document::cross_validate(&customs, std::slice::from_ref(&issued));
            assert!(
                cross_issues
                    .iter()
                    .any(|issue| issue.code == "cross_document_quantity_mismatch")
            );
            assert!(
                cross_issues
                    .iter()
                    .any(|issue| issue.code == "cross_document_amount_mismatch")
            );
            assert!(
                cross_issues
                    .iter()
                    .any(|issue| issue.code == "cross_document_hs_mismatch")
            );
            let mut packing_validation = issued.clone();
            packing_validation.id = "packing-validation".into();
            packing_validation.document_type = DocumentType::PackingList;
            packing_validation.payload.port_of_loading = "Shanghai".into();
            let mut bill_validation = packing_validation.clone();
            bill_validation.id = "bill-validation".into();
            bill_validation.document_type = DocumentType::BillOfLading;
            assert!(
                crate::document::cross_validate(
                    &bill_validation,
                    std::slice::from_ref(&packing_validation)
                )
                .is_empty()
            );
            bill_validation.payload.lines[0].gross_weight_kg += 1.0;
            bill_validation.payload.lines[0].cbm += 1.0;
            bill_validation.payload.port_of_loading = "Ningbo".into();
            let bill_issues = crate::document::cross_validate(
                &bill_validation,
                std::slice::from_ref(&packing_validation),
            );
            for code in [
                "cross_document_weight_mismatch",
                "cross_document_package_mismatch",
                "cross_document_transport_mismatch",
            ] {
                assert!(bill_issues.iter().any(|issue| issue.code == code));
            }
            let mut origin_validation = issued.clone();
            origin_validation.id = "origin-validation".into();
            origin_validation.document_type = DocumentType::CertificateOfOrigin;
            origin_validation.payload.origin_country = "Vietnam".into();
            let origin_issues =
                crate::document::cross_validate(&origin_validation, std::slice::from_ref(&issued));
            assert!(
                origin_issues
                    .iter()
                    .any(|issue| issue.code == "cross_document_origin_mismatch")
            );
            let mut inspection_validation = packing_validation.clone();
            inspection_validation.id = "inspection-validation".into();
            inspection_validation.document_type = DocumentType::InspectionCertificate;
            inspection_validation.payload.lines[0].net_weight_kg += 2.0;
            let inspection_issues = crate::document::cross_validate(
                &inspection_validation,
                std::slice::from_ref(&packing_validation),
            );
            assert!(
                inspection_issues
                    .iter()
                    .any(|issue| issue.code == "cross_document_weight_mismatch")
            );
            let mut fumigation_validation = packing_validation.clone();
            fumigation_validation.id = "fumigation-validation".into();
            fumigation_validation.document_type = DocumentType::FumigationCertificate;
            fumigation_validation.payload.lines[0].packages += 1;
            let fumigation_issues = crate::document::cross_validate(
                &fumigation_validation,
                std::slice::from_ref(&packing_validation),
            );
            assert!(
                fumigation_issues
                    .iter()
                    .any(|issue| issue.code == "cross_document_package_mismatch")
            );
            let mut beneficiary_validation = issued.clone();
            beneficiary_validation.id = "beneficiary-validation".into();
            beneficiary_validation.document_type = DocumentType::BeneficiaryCertificate;
            beneficiary_validation.payload.lines[0].unit_price_minor += 1;
            beneficiary_validation.payload.lines[0].amount_minor += 1;
            beneficiary_validation.payload.lines[0].hs_code = "DIFFERENT".into();
            let beneficiary_issues = crate::document::cross_validate(
                &beneficiary_validation,
                std::slice::from_ref(&issued),
            );
            for code in [
                "cross_document_amount_mismatch",
                "cross_document_hs_mismatch",
            ] {
                assert!(beneficiary_issues.iter().any(|issue| issue.code == code));
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
            assert_eq!(database.summary().unwrap().documents, 9);
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
        let cost_estimates = reopened.list_cost_estimates().unwrap();
        assert_eq!(cost_estimates[0].number, "CST-20260811-0001");
        assert_eq!(cost_estimates[0].lines.len(), 2);
        assert_eq!(cost_estimates[0].suggested_price_minor, 3_000);
        assert_eq!(reopened.list_partners().unwrap()[0].code, "FWD-001");
        assert_eq!(reopened.list_shipment_batches().unwrap()[0].lines.len(), 1);
        assert_eq!(
            reopened.list_payment_plans().unwrap()[0].status,
            PaymentStatus::Partial
        );
        let documents = reopened.list_documents().unwrap();
        assert_eq!(documents.len(), 10);
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
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::BillOfLading
                && document.status == DocumentStatus::Draft
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::InsurancePolicy
                && document.status == DocumentStatus::Draft
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::CertificateOfOrigin
                && document.status == DocumentStatus::Draft
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::InspectionCertificate
                && document.status == DocumentStatus::Draft
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::FumigationCertificate
                && document.status == DocumentStatus::Draft
        }));
        assert!(documents.iter().any(|document| {
            document.document_type == DocumentType::BeneficiaryCertificate
                && document.status == DocumentStatus::Draft
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
            "13"
        );
        drop(database);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn migrates_component_options_from_schema_v7() {
        let path = std::env::temp_dir().join(format!("tradedesk-v7-{}.db", Uuid::new_v4()));
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .pragma_update(None, "key", "test-password")
                .unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE workspace_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                     INSERT INTO workspace_meta(key, value) VALUES('schema_version', '7');
                     CREATE TABLE config_components (
                        id TEXT PRIMARY KEY, code TEXT NOT NULL UNIQUE, category TEXT NOT NULL,
                        name TEXT NOT NULL, specification TEXT NOT NULL DEFAULT '',
                        default_quantity REAL NOT NULL DEFAULT 1, unit TEXT NOT NULL,
                        unit_price_minor INTEGER NOT NULL DEFAULT 0, currency TEXT NOT NULL DEFAULT 'CNY',
                        brand TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '',
                        active INTEGER NOT NULL DEFAULT 1
                     );
                     INSERT INTO config_components(
                        id, code, category, name, specification, default_quantity, unit,
                        unit_price_minor, currency, brand, notes, active
                     ) VALUES(
                        'component-1', 'COMP-1', '冷却系统', '卧式远置散热器', '', 1, '套',
                        2850000, 'CNY', '华东冷却', '', 1
                     );",
                )
                .unwrap();
        }

        let database =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        let options = database.list_component_options().unwrap();
        assert_eq!(options.len(), 4);
        assert!(
            options
                .iter()
                .any(|option| option.kind == "category" && option.value == "冷却系统")
        );
        assert_eq!(
            database
                .connection
                .query_row(
                    "SELECT value FROM workspace_meta WHERE key = 'schema_version'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "13"
        );
        drop(database);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn localizes_configuration_and_reports_missing_terms() {
        let path = std::env::temp_dir().join(format!("tradedesk-i18n-{}.db", Uuid::new_v4()));
        let database =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        let component = database
            .save_config_component(ConfigComponentInput {
                id: None,
                code: "COMP-ENGINE-01".into(),
                category: "动力系统".into(),
                name: "天然气发动机".into(),
                specification: "K38N-G6".into(),
                default_quantity: 1.0,
                unit: "set".into(),
                unit_price_minor: 1_000_000,
                currency: "USD".into(),
                brand: "ACME".into(),
                notes: String::new(),
            })
            .unwrap();
        let configuration = database
            .save_configurable_product(ConfigurableProductInput {
                id: None,
                code: "CFG-GEN-01".into(),
                name: "天然气发电机组".into(),
                model: "K38N-G6".into(),
                currency: "USD".into(),
                exchange_rate: 0.14,
                exchange_rate_date: "2026-08-10".into(),
                notes: String::new(),
                lines: vec![ConfigurableProductLineInput {
                    component_id: component.id,
                    quantity: 1.0,
                    unit_price_minor: 1_000_000,
                }],
            })
            .unwrap();

        let (_, missing) = database
            .configuration_for_export(&configuration.id, "en")
            .unwrap();
        assert_eq!(missing.len(), 3);

        for (kind, source, translated) in [
            ("category", "动力系统", "Power System"),
            ("name", "天然气发动机", "Natural Gas Engine"),
            (
                "product_name",
                "天然气发电机组",
                "Natural Gas Generator Set",
            ),
        ] {
            let option = database
                .list_component_options()
                .unwrap()
                .into_iter()
                .find(|option| option.kind == kind && option.value == source)
                .unwrap();
            database
                .save_component_option_translation(ComponentOptionTranslationInput {
                    option_id: option.id,
                    language: "en".into(),
                    value: translated.into(),
                })
                .unwrap();
        }

        let (localized, missing) = database
            .configuration_for_export(&configuration.id, "en")
            .unwrap();
        assert!(missing.is_empty());
        assert_eq!(localized.name, "Natural Gas Generator Set");
        assert_eq!(localized.lines[0].category, "Power System");
        assert_eq!(localized.lines[0].name, "Natural Gas Engine");
        assert_eq!(
            database
                .list_component_options()
                .unwrap()
                .into_iter()
                .find(|option| option.kind == "name" && option.value == "天然气发动机")
                .unwrap()
                .translations
                .get("en"),
            Some(&"Natural Gas Engine".to_owned())
        );
        drop(database);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn stores_manual_exchange_rate_for_foreign_configuration() {
        let path = std::env::temp_dir().join(format!("tradedesk-rate-{}.db", Uuid::new_v4()));
        let database =
            EncryptedDatabase::open(&path, Zeroizing::new("test-password".to_owned())).unwrap();
        let component = database
            .save_config_component(ConfigComponentInput {
                id: None,
                code: "COMP-CNY-01".into(),
                category: "Power".into(),
                name: "Engine".into(),
                specification: "K38".into(),
                default_quantity: 1.0,
                unit: "set".into(),
                unit_price_minor: 100_000,
                currency: "CNY".into(),
                brand: "ACME".into(),
                notes: String::new(),
            })
            .unwrap();
        let configuration = database
            .save_configurable_product(ConfigurableProductInput {
                id: None,
                code: "CFG-USD-01".into(),
                name: "USD Quote".into(),
                model: "K38".into(),
                currency: "USD".into(),
                exchange_rate: 0.14,
                exchange_rate_date: "2026-08-10".into(),
                notes: String::new(),
                lines: vec![ConfigurableProductLineInput {
                    component_id: component.id,
                    quantity: 2.0,
                    unit_price_minor: 14_000,
                }],
            })
            .unwrap();
        assert_eq!(configuration.currency, "USD");
        assert_eq!(configuration.exchange_rate, 0.14);
        assert_eq!(configuration.exchange_rate_date, "2026-08-10");
        assert_eq!(configuration.total_amount_minor, 28_000);
        drop(database);
        let _ = std::fs::remove_file(&path);
    }
}
