use crate::database::columns::PubkeyTypeString;
use crate::global::enums::dex_type::DexType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pools")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: PubkeyTypeString,
    pub name: String,
    pub dex_type: DexType,
    pub base_mint: PubkeyTypeString,
    pub quote_mint: PubkeyTypeString,
    #[sea_orm(column_type = "JsonBinary")]
    pub description: PoolRecordDescriptor,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PoolRecordDescriptor {
    pub base_repr: String,
    pub quote_repr: String,
}

impl Eq for Model {}

impl Hash for Model {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_pool_record_descriptor_serialization() {
        let descriptor = PoolRecordDescriptor {
            base_repr: "TNS".to_string(),
            quote_repr: "SOL".to_string(),
        };

        let json = serde_json::to_string(&descriptor).unwrap();
        println!("Serialized JSON: {}", json);

        let expected = r#"{"base_repr":"TNS","quote_repr":"SOL"}"#;
        assert_eq!(json, expected);

        let deserialized: PoolRecordDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, descriptor);
    }

    #[test]
    fn test_pool_record_descriptor_from_json_with_byte_array() {
        let json_with_bytes = r#"{"base": [3, 0, 123, 71, 65, 72, 195, 12, 199, 132, 151, 245, 245, 38, 215, 124, 207, 130, 45, 35, 107, 31, 13, 167, 164, 111, 148, 103, 30, 148, 249, 191], "quote": [6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53, 218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1], "base_repr": "TNS", "quote_repr": "SOL"}"#;

        let result = serde_json::from_str::<PoolRecordDescriptor>(json_with_bytes);
        assert!(
            result.is_err(),
            "Should fail to deserialize byte arrays as strings"
        );
    }
}
