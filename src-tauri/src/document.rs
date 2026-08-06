use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Issued,
    Voided,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    pub document_type: String,
    pub document_number: String,
    pub business_case_id: String,
    pub version: u32,
    pub status: DocumentStatus,
    pub payload: serde_json::Value,
}

impl DocumentSnapshot {
    pub fn validate_for_issue(&self) -> Result<(), &'static str> {
        if self.document_number.trim().is_empty() {
            return Err("document number is required");
        }
        if self.version == 0 {
            return Err("document version must start at one");
        }
        Ok(())
    }
}
