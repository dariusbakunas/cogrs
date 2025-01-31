use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "A self-contained automation module")]
pub struct ModuleArgs {
    /// Print the module's input schema in JSON format.
    #[arg(long)]
    pub schema: bool,

    /// Input data in JSON format passed to the module.
    #[arg(long, value_name = "JSON")]
    pub inputs: Option<String>,
}
