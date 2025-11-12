use crate::commands::{chat, config, model, setup};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tt",
    about = "Transform natural language into shell commands.",
    version = crate::version::TT_VERSION
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The question to ask the selected provider (if no subcommand is provided)
    #[arg(trailing_var_arg = true)]
    question: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure the provider, API key, and default model
    Setup,

    /// Show current configuration
    Config,

    /// Change the default model
    Model,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Setup) => setup::run().await?,
        Some(Commands::Config) => config::show()?,
        Some(Commands::Model) => model::change().await?,
        None => {
            if cli.question.is_empty() {
                anyhow::bail!("Please provide a question or run 'tt setup' to configure.");
            }
            let question = cli.question.join(" ");
            chat::run(&question).await?;
        }
    }

    Ok(())
}
