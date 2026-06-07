use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaField {
    pub name: String,
    pub size: u32,
    pub field_type: String, // "int", "float", "string", "binary"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DynamicSchema {
    pub fields: Vec<SchemaField>,
}

impl DynamicSchema {
    pub fn new(fields: Vec<SchemaField>) -> Self {
        Self { fields }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.fields).unwrap_or_else(|_| "[]".to_string())
    }

    /// Calculates a stable 64-bit hash (FNV-1a) for the schema ID based on its JSON representation.
    pub fn schema_id(&self) -> i64 {
        let json = self.to_json();
        let mut hash: u64 = 14695981039346656037;
        for b in json.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash as i64
    }
}
