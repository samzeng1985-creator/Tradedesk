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
    pub product_id: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
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
