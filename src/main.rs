use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use myeon::cli;
use myeon::cli::{Cli, Commands};
use myeon::colours;
use myeon::data::{Priority, Task, TaskStatus};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{error::Error, io};

/// myeon Theme - Derived from ilseon/ui/theme/Colour.kt
const BG_DEEP: Color = Color::Rgb(18, 18, 18); // DarkGrey
const FG_PRIMARY: Color = Color::Rgb(224, 224, 224); // TextPrimary
const FG_MUTED: Color = Color::Rgb(176, 176, 176); // TextSecondary
const BORDER_ACTIVE: Color = Color::Rgb(90, 155, 128); // MutedTeal
const BORDER_QUIET: Color = Color::Rgb(31, 31, 31); // BorderQuiet
const ACCENT_URGENT: Color = Color::Rgb(179, 95, 95); // MutedRed (StatusUrgent)

struct App {
    column_index: usize,
    selected_task_index: usize,
    all_tasks: Vec<Task>,
    current_context: String, // For filtering (e.g., "All" or "Work")
}

impl App {
    fn new() -> App {
        let tasks = vec![
            Task {
                id: uuid::Uuid::new_v4(),
                title: "Implement basic TUI".to_string(),
                description: None,
                status: TaskStatus::Done,
                priority: Priority::High,
                context: "Work".to_string(),
                created_at: chrono::Utc::now(),
            },
            Task {
                id: uuid::Uuid::new_v4(),
                title: "Fix rendering bugs".to_string(),
                description: Some("Fix all the things!".to_string()),
                status: TaskStatus::Doing,
                priority: Priority::Medium,
                context: "Work".to_string(),
                created_at: chrono::Utc::now(),
            },
            Task {
                id: uuid::Uuid::new_v4(),
                title: "Add task creation".to_string(),
                description: None,
                status: TaskStatus::Todo,
                priority: Priority::Low,
                context: "Work".to_string(),
                created_at: chrono::Utc::now(),
            },
            Task {
                id: uuid::Uuid::new_v4(),
                title: "Buy groceries".to_string(),
                description: Some("Milk, Bread, Cheese".to_string()),
                status: TaskStatus::Todo,
                priority: Priority::High,
                context: "Personal".to_string(),
                created_at: chrono::Utc::now(),
            },
        ];
        App {
            column_index: 0,
            selected_task_index: 0,
            all_tasks: tasks,
            current_context: "All".to_string(),
        }
    }

    // Filter tasks based on status for the UI columns
    fn tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.all_tasks
            .iter()
            .filter(|t| t.status == status)
            .filter(|t| self.current_context == "All" || t.context == self.current_context)
            .collect()
    }
}

fn main() {
    let cli = cli::Cli::parse();
    if let Err(e) = run(cli) {
        colours::error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}

// The main logic function, which takes the parsed CLI commands
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
            if status.updated() {
                println!("Successfully updated myeon!");
            } else {
                println!("myeon is already up to date.");
            }
            Ok(())
        }
        None => {
            // Setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            // Create app and run loop
            let app = App::new();
            let res = run_app(&mut terminal, app);

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            if let Err(err) = res {
                println!("{:?}", err)
            }

            Ok(())
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app)).expect("TODO: panic message");

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') | KeyCode::Left => {
                    if app.column_index > 0 {
                        app.column_index -= 1;
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if app.column_index < 2 {
                        app.column_index += 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    // --- Header ---
    let header = Paragraph::new(" myeon | h/l to move • q to quit ")
        .style(Style::default().fg(FG_MUTED))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(FG_MUTED)),
        );
    f.render_widget(header, chunks[0]);

    // --- Columns ---
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    let todo_tasks = app.tasks_by_status(TaskStatus::Todo);
    let doing_tasks = app.tasks_by_status(TaskStatus::Doing);
    let done_tasks = app.tasks_by_status(TaskStatus::Done);

    render_column(f, columns[0], "To Do", &todo_tasks, app.column_index == 0);
    render_column(f, columns[1], "Doing", &doing_tasks, app.column_index == 1);
    render_column(f, columns[2], "Done", &done_tasks, app.column_index == 2);
}

fn render_column(f: &mut Frame, area: Rect, title: &str, items: &[&Task], is_active: bool) {
    let border_color = if is_active {
        BORDER_ACTIVE
    } else {
        BORDER_QUIET
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(format!(" • {}", i.title)).style(Style::default().fg(FG_PRIMARY)))
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title))
            .border_style(Style::default().fg(border_color)),
    );

    f.render_widget(list, area);
}
