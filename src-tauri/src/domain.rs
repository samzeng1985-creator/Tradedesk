use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    Quotation,
    Order,
    Purchase,
    Production,
    Shipment,
    Documents,
}

impl PipelineStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quotation => "quotation",
            Self::Order => "order",
            Self::Purchase => "purchase",
            Self::Production => "production",
            Self::Shipment => "shipment",
            Self::Documents => "documents",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "quotation" => Some(Self::Quotation),
            "order" => Some(Self::Order),
            "purchase" => Some(Self::Purchase),
            "production" => Some(Self::Production),
            "shipment" => Some(Self::Shipment),
            "documents" => Some(Self::Documents),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub company_name: String,
    pub encrypted: bool,
    pub products: u64,
    pub customers: u64,
    pub suppliers: u64,
    pub active_cases: u64,
    pub purchase_orders: u64,
    pub production_risks: u64,
    pub documents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    CommercialQuotation,
    ProformaInvoice,
    CommercialInvoice,
    PackingList,
    TradeContract,
}

impl DocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CommercialQuotation => "commercial_quotation",
            Self::ProformaInvoice => "proforma_invoice",
            Self::CommercialInvoice => "commercial_invoice",
            Self::PackingList => "packing_list",
            Self::TradeContract => "trade_contract",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "commercial_quotation" => Some(Self::CommercialQuotation),
            "proforma_invoice" => Some(Self::ProformaInvoice),
            "commercial_invoice" => Some(Self::CommercialInvoice),
            "packing_list" => Some(Self::PackingList),
            "trade_contract" => Some(Self::TradeContract),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Issued,
    Voided,
}

impl DocumentStatus {
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "issued" => Some(Self::Issued),
            "voided" => Some(Self::Voided),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentValidationIssue {
    pub severity: ValidationSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLineSnapshot {
    pub product_id: String,
    pub sku: String,
    pub description: String,
    pub model: String,
    pub hs_code: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price_minor: i64,
    pub amount_minor: i64,
    pub packages: i64,
    pub package_type: String,
    pub net_weight_kg: f64,
    pub gross_weight_kg: f64,
    pub cbm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPayload {
    pub seller: String,
    pub seller_address: String,
    pub buyer: String,
    pub buyer_address: String,
    pub origin_country: String,
    pub destination_country: String,
    pub port_of_loading: String,
    pub port_of_discharge: String,
    pub incoterm: String,
    pub payment_terms: String,
    pub shipment_date: String,
    pub po_reference: String,
    #[serde(default)]
    pub valid_until: String,
    #[serde(default)]
    pub discount_minor: i64,
    pub bank_details: String,
    pub notes: String,
    pub declaration: String,
    pub contract_terms: String,
    pub lines: Vec<DocumentLineSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeDocument {
    pub id: String,
    pub document_type: DocumentType,
    pub number: String,
    pub business_case_id: String,
    pub business_case_number: String,
    pub customer_name: String,
    pub version: u32,
    pub status: DocumentStatus,
    pub language: String,
    pub issue_date: String,
    pub currency: String,
    pub template_version: String,
    pub payload: DocumentPayload,
    pub validation_issues: Vec<DocumentValidationIssue>,
    pub void_reason: String,
    pub pdf_path: String,
    pub pdf_sha256: String,
    pub exported_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentInput {
    pub business_case_id: String,
    pub document_type: DocumentType,
    pub number: String,
    pub language: String,
    pub issue_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertDocumentInput {
    pub source_document_id: String,
    pub target_document_type: DocumentType,
    pub number: String,
    pub language: String,
    pub issue_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentInput {
    pub id: String,
    pub number: String,
    pub language: String,
    pub issue_date: String,
    pub payload: DocumentPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentExportResult {
    pub path: String,
    pub sha256: String,
    pub exported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub sku: String,
    pub name_zh: String,
    pub name_en: String,
    pub model: String,
    pub hs_code: String,
    pub unit: String,
    pub gross_weight_kg: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInput {
    pub id: Option<String>,
    pub sku: String,
    pub name_zh: String,
    pub name_en: String,
    pub model: String,
    pub hs_code: String,
    pub unit: String,
    pub gross_weight_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub id: String,
    pub code: String,
    pub legal_name: String,
    pub market: String,
    pub currency: String,
    pub payment_terms: String,
    pub address: String,
    pub shipping_address: String,
    pub billing_address: String,
    pub purchase_intent: String,
    pub customer_analysis: String,
    pub strengths: String,
    pub weaknesses: String,
    pub contacts: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInput {
    pub id: Option<String>,
    pub code: String,
    pub legal_name: String,
    pub market: String,
    pub currency: String,
    pub payment_terms: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub shipping_address: String,
    #[serde(default)]
    pub billing_address: String,
    #[serde(default)]
    pub purchase_intent: String,
    #[serde(default)]
    pub customer_analysis: String,
    #[serde(default)]
    pub strengths: String,
    #[serde(default)]
    pub weaknesses: String,
    #[serde(default)]
    pub contacts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supplier {
    pub id: String,
    pub code: String,
    pub legal_name: String,
    pub lead_time_days: i64,
    pub on_time_rate: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInput {
    pub id: Option<String>,
    pub code: String,
    pub legal_name: String,
    pub lead_time_days: i64,
    pub on_time_rate: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessCaseLine {
    pub id: String,
    pub product_id: String,
    pub sku: String,
    pub name_zh: String,
    pub name_en: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price_minor: i64,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessCaseLineInput {
    pub id: Option<String>,
    pub product_id: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseStatus {
    Draft,
    PendingConfirmation,
    Confirmed,
    InProduction,
    ReadyToShip,
    Completed,
    Cancelled,
}

impl PurchaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::PendingConfirmation => "pending_confirmation",
            Self::Confirmed => "confirmed",
            Self::InProduction => "in_production",
            Self::ReadyToShip => "ready_to_ship",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "pending_confirmation" => Some(Self::PendingConfirmation),
            "confirmed" => Some(Self::Confirmed),
            "in_production" => Some(Self::InProduction),
            "ready_to_ship" => Some(Self::ReadyToShip),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

impl MilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Blocked => "blocked",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMilestone {
    pub id: String,
    pub purchase_order_line_id: String,
    pub stage: String,
    pub label: String,
    pub planned_date: String,
    pub actual_date: String,
    pub progress: i64,
    pub completed_quantity: f64,
    pub status: MilestoneStatus,
    pub issue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMilestoneInput {
    pub id: String,
    pub planned_date: String,
    pub actual_date: String,
    pub progress: i64,
    pub completed_quantity: f64,
    pub status: MilestoneStatus,
    pub issue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderLine {
    pub id: String,
    pub source_case_line_id: String,
    pub product_id: String,
    pub sku: String,
    pub name_zh: String,
    pub name_en: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_cost_minor: i64,
    pub amount_minor: i64,
    pub milestones: Vec<ProductionMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderLineInput {
    pub source_case_line_id: String,
    pub quantity: f64,
    pub unit_cost_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrder {
    pub id: String,
    pub number: String,
    pub business_case_id: String,
    pub business_case_number: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: PurchaseStatus,
    pub currency: String,
    pub expected_date: String,
    pub notes: String,
    pub total_amount_minor: i64,
    pub completed_quantity: f64,
    pub ready_quantity: f64,
    pub lines: Vec<PurchaseOrderLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderInput {
    pub number: String,
    pub business_case_id: String,
    pub supplier_id: String,
    pub currency: String,
    pub expected_date: String,
    pub notes: String,
    pub lines: Vec<PurchaseOrderLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessCase {
    pub id: String,
    pub number: String,
    pub customer_id: String,
    pub customer_name: String,
    pub stage: PipelineStage,
    pub currency: String,
    pub incoterm: String,
    pub payment_terms: String,
    pub shipment_date: String,
    pub notes: String,
    pub total_amount_minor: i64,
    pub lines: Vec<BusinessCaseLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessCaseInput {
    pub id: Option<String>,
    pub number: String,
    pub customer_id: String,
    pub stage: PipelineStage,
    pub currency: String,
    pub incoterm: String,
    pub payment_terms: String,
    pub shipment_date: String,
    pub notes: String,
    pub lines: Vec<BusinessCaseLineInput>,
}
