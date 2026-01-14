use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "myeon",
    author = "Claes Adamsson @cladam",
    version,
    about = "myeon is a minimalist, keyboard-driven TUI Kanban board",
    long_about = None,
    after_help = "KEYBINDINGS:\n  h/j/k/l    Move focus across tasks and columns\n  a          Quick-capture a new idea\n  e          Edit a task\n  c          Change Context (cycle Work/Personal/etc.)\n  Enter      Move the task forward\n  Backspace  Move the task backward\n  d          Delete a task\n  q          Quit"
)]
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
