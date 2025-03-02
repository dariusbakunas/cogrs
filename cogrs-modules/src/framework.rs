use anyhow::Result;
use serde_json::Value;

/// The core Module Trait for all modules.
pub trait Module {
    /// Define the JSON Schema for the module.
    fn schema() -> &'static str;

    /// Logic to execute the module after validation.
    fn run(inputs: Value) -> Result<()>;
}

#[macro_export]
macro_rules! define_module {
    ($module_type:ty) => {
        fn main() -> Result<()> {
            use anyhow::{anyhow, Result};
            use clap::Parser;
            use cogrs_schema::validation::validate_input;
            use serde_json::Value;
            use $crate::cli::ModuleArgs;

            let args = ModuleArgs::parse();

            if args.schema {
                println!("{}", <$module_type as $crate::framework::Module>::schema());
                return Ok(());
            }

            let input_str = args
                .inputs
                .ok_or_else(|| anyhow!("You must provide input data using --inputs"))?;

            let inputs: Value = serde_json::from_str(&input_str)
                .map_err(|e| anyhow!("Failed to parse inputs as JSON: {}", e))?;

            validate_input(
                <$module_type as $crate::framework::Module>::schema(),
                &inputs,
            )?;

            <$module_type as $crate::framework::Module>::run(inputs)
        }
    };
}
