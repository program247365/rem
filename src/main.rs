use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use std::io::IsTerminal;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod eventkit;
mod logging;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    // Check if we're running in a proper terminal
    if !std::io::IsTerminal::is_terminal(&std::io::stderr()) {
        eprintln!("Error: rem must be run in a terminal environment.");
        eprintln!("This TUI application requires a proper TTY to function.");
        eprintln!("Please run rem directly in your terminal, not through a pipe or redirect.");
        std::process::exit(1);
    }

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
