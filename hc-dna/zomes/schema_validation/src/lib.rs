use hdk::prelude::*;
use lazy_static::lazy_static;
use jsonschema_valid::{schemas, Config};
use serde_json::Value;
use chrono::{DateTime, Utc};

mod entries;

use entries::*;

// TODOs
// schema json validate entry
// - field not too big, max length

entry_defs![
    Expression::entry_def(),
    Path::entry_def()
];

#[derive(SerializedBytes, Serialize, Deserialize, Clone, Debug)]
pub struct CreateExpressionInput {
    pub data: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub proof: ExpressionProof,
}

#[hdk_extern]
pub fn create_expression(input: CreateExpressionInput) -> ExternResult<EntryHash> {
    let CreateExpressionInput { data, author, timestamp, proof } = input;

    let schema: Value = serde_json::from_str(&EXPRESSION_SCHEMA)
        .map_err(|e| WasmError::Host(e.to_string()))?;
    let cfg = Config::from_schema(&schema, Some(schemas::Draft::Draft7))
        .map_err(|e| WasmError::Host(e.to_string()))?;
    assert!(cfg.validate_schema().is_ok());
    
    let data_json: Value = serde_json::from_str(&data)
        .map_err(|e| WasmError::Host(e.to_string()))?;
    assert!(cfg.validate(&data_json).is_ok());

    let expression = Expression {
        data: data_json,
        author,
        timestamp,
        proof,
    };

    let entry_hash = hash_entry(&expression)?;
    let _header_hash = create_entry(&expression)?;

    hc_time_index::index_entry(expression.author.clone(), expression.clone(), LinkTag::new("expression"))
        .map_err(|e| WasmError::Host(e.to_string()))?;
    
    Ok(entry_hash)
}

#[derive(SerializedBytes, Serialize, Deserialize, Debug)]
pub struct Properties {
    pub expression_data_schema: String,
}

lazy_static! {
    pub static ref EXPRESSION_SCHEMA: String = {
        let host_dna_config = zome_info()
            .expect("Could not get zome configuration.")
            .properties;
        let properties = Properties::try_from(host_dna_config)
            .expect("Could not convert zome dna properties to Properties.");
        properties.expression_data_schema
    };
}
