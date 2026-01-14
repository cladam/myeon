use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use myeon::{
    app::App,
    cli::{Cli, Commands},
    colours, input, ui,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io};

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        colours::error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Some(Commands::Update) => {
            println!("--- Checking for updates ---");
            let status = self_update::backends::github::Update::configure()
                .repo_owner("cladam")
                .repo_name("myeon")
                .bin_name("myeon")
                .show_download_progress(true)
                .current_version(self_update::cargo_crate_version!())
                .build()?
                .update()?;
            println!("Update status: `{}`!", status.version());
            Ok(())
        }
        None => run_tui(),
    }
}

fn run_tui() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        if input::handle_input(&mut app)? {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
