#[allow(unused)]
use clap::Parser;
use spin_trigger::cli::TriggerExecutorCommand;
use spin_trigger_messaging::MessagingTrigger;
use std::io::IsTerminal;

type Command = TriggerExecutorCommand<MessagingTrigger>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    let trigger = Command::parse();
    trigger.run().await
}
