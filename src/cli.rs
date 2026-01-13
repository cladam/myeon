use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "myeon",
    author = "Claes Adamsson @cladam",
    version,
    about = "myeon is a minimalist, keyboard-driven TUI Kanban board",
    long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Update myeon to the latest version.
    #[command(name = "update", hide = true)] // Hidden from help
    Update,
}
