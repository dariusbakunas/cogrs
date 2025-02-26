use anyhow::{bail, Context, Result};
use jsonschema::ValidationError;
use serde_json::Value;

/// Validate the input data against a JSON Schema.
pub fn validate_input(schema: &str, input: &Value) -> Result<()> {
    let compiled_schema =
        &serde_json::from_str::<Value>(schema).context("Failed to parse plugn schema")?;
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

#[cfg(test)]
mod tests {
    use super::validate_input;

    #[test]
    fn test_valid_input() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }"#;
        let input = serde_json::from_str(r#"{"name": "Cogrs"}"#).unwrap();
        assert!(validate_input(schema, &input).is_ok());
    }

    #[test]
    fn test_invalid_input() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }"#;
        let input = serde_json::from_str(r#"{"age": 30}"#).unwrap();
        assert!(validate_input(schema, &input).is_err());
    }

    #[test]
    fn test_unknown_property() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "additionalProperties": false,
            "required": ["name"]
        }"#;
        let input = serde_json::from_str(r#"{"name": "Cogrs", "test": "invalid"}"#).unwrap();
        assert!(validate_input(schema, &input).is_err());
    }
}
