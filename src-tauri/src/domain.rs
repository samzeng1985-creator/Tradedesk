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
    #[serde(default)]
    pub recovery_key: String,
    #[serde(default)]
    pub recovery_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default)]
    pub entity_label: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInput {
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default)]
    pub entity_label: String,
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDraft {
    pub input: SaveDocumentInput,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub company_name: String,
    pub logo_data_url: String,
    pub signature_data_url: String,
    pub signing_asset_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanySigningAsset {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub data_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyRecord {
    pub id: String,
    pub company_name: String,
    pub logo_data_url: String,
    pub signing_assets: Vec<CompanySigningAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyRegistry {
    pub default_company_id: String,
    pub companies: Vec<CompanyRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    CommercialQuotation,
    ProformaInvoice,
    CommercialInvoice,
    PackingList,
    TradeContract,
    ShippingMarks,
    ShipperInstruction,
    CustomsDeclaration,
    BillOfLading,
    InsurancePolicy,
    CertificateOfOrigin,
    InspectionCertificate,
    FumigationCertificate,
    BeneficiaryCertificate,
}

impl DocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CommercialQuotation => "commercial_quotation",
            Self::ProformaInvoice => "proforma_invoice",
            Self::CommercialInvoice => "commercial_invoice",
            Self::PackingList => "packing_list",
            Self::TradeContract => "trade_contract",
            Self::ShippingMarks => "shipping_marks",
            Self::ShipperInstruction => "shipper_instruction",
            Self::CustomsDeclaration => "customs_declaration",
            Self::BillOfLading => "bill_of_lading",
            Self::InsurancePolicy => "insurance_policy",
            Self::CertificateOfOrigin => "certificate_of_origin",
            Self::InspectionCertificate => "inspection_certificate",
            Self::FumigationCertificate => "fumigation_certificate",
            Self::BeneficiaryCertificate => "beneficiary_certificate",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "commercial_quotation" => Some(Self::CommercialQuotation),
            "proforma_invoice" => Some(Self::ProformaInvoice),
            "commercial_invoice" => Some(Self::CommercialInvoice),
            "packing_list" => Some(Self::PackingList),
            "trade_contract" => Some(Self::TradeContract),
            "shipping_marks" => Some(Self::ShippingMarks),
            "shipper_instruction" => Some(Self::ShipperInstruction),
            "customs_declaration" => Some(Self::CustomsDeclaration),
            "bill_of_lading" => Some(Self::BillOfLading),
            "insurance_policy" => Some(Self::InsurancePolicy),
            "certificate_of_origin" => Some(Self::CertificateOfOrigin),
            "inspection_certificate" => Some(Self::InspectionCertificate),
            "fumigation_certificate" => Some(Self::FumigationCertificate),
            "beneficiary_certificate" => Some(Self::BeneficiaryCertificate),
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
    #[serde(default)]
    pub shipping_marks: String,
    #[serde(default)]
    pub transport_mode: String,
    #[serde(default)]
    pub vessel_voyage: String,
    #[serde(default)]
    pub booking_reference: String,
    #[serde(default)]
    pub freight_terms: String,
    #[serde(default)]
    pub bill_of_lading_type: String,
    #[serde(default)]
    pub customs_supervision_code: String,
    #[serde(default)]
    pub customs_declaration_elements: String,
    #[serde(default)]
    pub notify_party: String,
    #[serde(default)]
    pub notify_party_address: String,
    #[serde(default)]
    pub carrier: String,
    #[serde(default)]
    pub bill_of_lading_number: String,
    #[serde(default)]
    pub place_of_receipt: String,
    #[serde(default)]
    pub place_of_delivery: String,
    #[serde(default)]
    pub container_numbers: String,
    #[serde(default)]
    pub seal_numbers: String,
    #[serde(default)]
    pub insurance_company: String,
    #[serde(default)]
    pub policy_number: String,
    #[serde(default)]
    pub insured_value_minor: i64,
    #[serde(default)]
    pub insurance_markup_percent: f64,
    #[serde(default)]
    pub premium_rate_percent: f64,
    #[serde(default)]
    pub premium_minor: i64,
    #[serde(default)]
    pub insurance_coverage: String,
    #[serde(default)]
    pub claims_payable_at: String,
    #[serde(default)]
    pub certificate_number: String,
    #[serde(default)]
    pub certificate_type: String,
    #[serde(default)]
    pub certification_authority: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub manufacturer_address: String,
    #[serde(default)]
    pub batch_number: String,
    #[serde(default)]
    pub inspection_standard: String,
    #[serde(default)]
    pub inspection_date: String,
    #[serde(default)]
    pub inspection_place: String,
    #[serde(default)]
    pub inspection_result: String,
    #[serde(default)]
    pub fumigation_agent: String,
    #[serde(default)]
    pub fumigation_method: String,
    #[serde(default)]
    pub fumigation_temperature_celsius: f64,
    #[serde(default)]
    pub fumigation_duration_hours: f64,
    #[serde(default)]
    pub fumigation_date: String,
    #[serde(default)]
    pub fumigation_place: String,
    #[serde(default)]
    pub fumigation_operator: String,
    #[serde(default)]
    pub fumigation_license_number: String,
    #[serde(default)]
    pub letter_of_credit_number: String,
    #[serde(default)]
    pub issuing_bank: String,
    #[serde(default)]
    pub letter_of_credit_issue_date: String,
    #[serde(default)]
    pub letter_of_credit_expiry_date: String,
    #[serde(default)]
    pub presentation_deadline: String,
    #[serde(default)]
    pub beneficiary_certificate_type: String,
    #[serde(default)]
    pub beneficiary_statement: String,
    #[serde(default)]
    pub letter_of_credit_terms: String,
    #[serde(default)]
    pub required_documents: String,
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
pub struct ConfigComponent {
    pub id: String,
    pub code: String,
    pub category: String,
    pub name: String,
    pub specification: String,
    pub default_quantity: f64,
    pub unit: String,
    pub unit_price_minor: i64,
    pub currency: String,
    pub brand: String,
    pub notes: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigComponentInput {
    pub id: Option<String>,
    pub code: String,
    pub category: String,
    pub name: String,
    pub specification: String,
    pub default_quantity: f64,
    pub unit: String,
    pub unit_price_minor: i64,
    pub currency: String,
    pub brand: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOption {
    pub id: String,
    pub kind: String,
    pub value: String,
    pub active: bool,
    pub translations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOptionInput {
    pub id: Option<String>,
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOptionTranslationInput {
    pub option_id: String,
    pub language: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableProductLine {
    pub id: String,
    pub component_id: String,
    pub category: String,
    pub name: String,
    pub specification: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price_minor: i64,
    pub brand: String,
    pub notes: String,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableProductLineInput {
    pub component_id: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableProduct {
    pub id: String,
    pub code: String,
    pub name: String,
    pub model: String,
    pub currency: String,
    pub exchange_rate: f64,
    pub exchange_rate_date: String,
    pub notes: String,
    pub total_amount_minor: i64,
    pub active: bool,
    pub lines: Vec<ConfigurableProductLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableProductInput {
    pub id: Option<String>,
    pub code: String,
    pub name: String,
    pub model: String,
    pub currency: String,
    pub exchange_rate: f64,
    pub exchange_rate_date: String,
    pub notes: String,
    pub lines: Vec<ConfigurableProductLineInput>,
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
    pub source_type: String,
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
    #[serde(default = "default_business_line_source")]
    pub source_type: String,
    pub product_id: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
}

fn default_business_line_source() -> String {
    "product".to_owned()
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
    pub exchange_rate: f64,
    pub exchange_rate_date: String,
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
    pub exchange_rate: f64,
    pub exchange_rate_date: String,
    pub expected_date: String,
    pub notes: String,
    pub lines: Vec<PurchaseOrderLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderUpdateInput {
    pub id: String,
    pub supplier_id: String,
    pub currency: String,
    pub exchange_rate: f64,
    pub exchange_rate_date: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEstimateLine {
    pub id: String,
    pub category: String,
    pub description: String,
    pub specification: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_cost_minor: i64,
    pub amount_minor: i64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEstimateLineInput {
    pub id: Option<String>,
    pub category: String,
    pub description: String,
    pub specification: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_cost_minor: i64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEstimate {
    pub id: String,
    pub number: String,
    pub business_case_id: String,
    pub business_case_number: String,
    pub customer_name: String,
    pub currency: String,
    pub target_margin_bps: i64,
    pub notes: String,
    pub total_cost_minor: i64,
    pub suggested_price_minor: i64,
    pub updated_at: String,
    pub lines: Vec<CostEstimateLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEstimateInput {
    pub id: Option<String>,
    pub number: String,
    pub business_case_id: String,
    pub target_margin_bps: i64,
    pub notes: String,
    pub lines: Vec<CostEstimateLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    pub id: String,
    pub code: String,
    pub legal_name: String,
    pub partner_type: String,
    pub contact: String,
    pub address: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerInput {
    pub id: Option<String>,
    pub code: String,
    pub legal_name: String,
    pub partner_type: String,
    pub contact: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShipmentStatus {
    Planned,
    Booked,
    Shipped,
    Delivered,
    Cancelled,
}

impl ShipmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Booked => "booked",
            Self::Shipped => "shipped",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "planned" => Some(Self::Planned),
            "booked" => Some(Self::Booked),
            "shipped" => Some(Self::Shipped),
            "delivered" => Some(Self::Delivered),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentLine {
    pub id: String,
    pub business_case_line_id: String,
    pub sku: String,
    pub product_name: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentLineInput {
    pub business_case_line_id: String,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentBatch {
    pub id: String,
    pub number: String,
    pub business_case_id: String,
    pub business_case_number: String,
    pub partner_id: String,
    pub partner_name: String,
    pub status: ShipmentStatus,
    pub planned_date: String,
    pub actual_date: String,
    pub tracking_number: String,
    pub notes: String,
    pub lines: Vec<ShipmentLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentBatchInput {
    pub id: Option<String>,
    pub number: String,
    pub business_case_id: String,
    pub partner_id: String,
    pub status: ShipmentStatus,
    pub planned_date: String,
    pub actual_date: String,
    pub tracking_number: String,
    pub notes: String,
    pub lines: Vec<ShipmentLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Planned,
    Partial,
    Received,
    Cancelled,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Partial => "partial",
            Self::Received => "received",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "planned" => Some(Self::Planned),
            "partial" => Some(Self::Partial),
            "received" => Some(Self::Received),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPlan {
    pub id: String,
    pub number: String,
    pub business_case_id: String,
    pub business_case_number: String,
    pub payment_type: String,
    pub due_date: String,
    pub currency: String,
    pub amount_minor: i64,
    pub received_amount_minor: i64,
    pub received_date: String,
    pub status: PaymentStatus,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPlanInput {
    pub id: Option<String>,
    pub number: String,
    pub business_case_id: String,
    pub payment_type: String,
    pub due_date: String,
    pub amount_minor: i64,
    pub received_amount_minor: i64,
    pub received_date: String,
    pub status: PaymentStatus,
    pub notes: String,
}
