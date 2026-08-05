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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub company_name: String,
    pub encrypted: bool,
    pub products: u64,
    pub customers: u64,
    pub suppliers: u64,
    pub active_cases: u64,
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
