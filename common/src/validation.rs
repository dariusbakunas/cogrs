use anyhow::{bail, Result};
use jsonschema::ValidationError;
use serde_json::Value;

/// Validate the input data against a JSON Schema.
pub fn validate_input(schema: &str, input: &Value) -> Result<()> {
    let compiled_schema = &serde_json::from_str::<Value>(schema)?;
    let validator = jsonschema::validator_for(compiled_schema)?;

    let err_message = validator
        .iter_errors(input)
        .map(|e: ValidationError| format!(" - {}", e))
        .collect::<Vec<String>>()
        .join("\n");

    if !err_message.is_empty() {
        bail!("Input validation failed:\n{}", err_message);
    }

    Ok(())
}
