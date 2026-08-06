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
    pub fn next(&self) -> Self {
        match self {
            Self::Quotation => Self::Order,
            Self::Order => Self::Purchase,
            Self::Purchase => Self::Production,
            Self::Production => Self::Shipment,
            Self::Shipment | Self::Documents => Self::Documents,
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCase {
    pub id: String,
    pub number: String,
    pub customer_id: String,
    pub stage: PipelineStage,
    pub currency: String,
    pub sales_amount_minor: i64,
    pub purchase_amount_minor: i64,
}

#[cfg(test)]
mod tests {
    use super::PipelineStage;

    #[test]
    fn pipeline_stops_at_documents() {
        assert_eq!(PipelineStage::Shipment.next(), PipelineStage::Documents);
        assert_eq!(PipelineStage::Documents.next(), PipelineStage::Documents);
    }
}
