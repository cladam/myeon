use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};

/// myeon Theme - Derived from ilseon/ui/theme/Colour.kt
const BG_DEEP: Color = Color::Rgb(18, 18, 18);        // DarkGrey
const FG_PRIMARY: Color = Color::Rgb(224, 224, 224);  // TextPrimary
const FG_MUTED: Color = Color::Rgb(176, 176, 176);    // TextSecondary
const BORDER_ACTIVE: Color = Color::Rgb(90, 155, 128);// MutedTeal
const BORDER_QUIET: Color = Color::Rgb(31, 31, 31);   // BorderQuiet
const ACCENT_URGENT: Color = Color::Rgb(179, 95, 95); // MutedRed (StatusUrgent)

struct App {
    column_index: usize,
    todo: Vec<String>,
    doing: Vec<String>,
    done: Vec<String>,
}

impl App {
    fn new() -> App {
        App {
            column_index: 0,
            todo: vec!["Refactor TUI".to_string(), "Add Ilseon colors".to_string()],
            doing: vec!["Build boilerplate".to_string()],
            done: vec!["Choose name".to_string()],
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app)).expect("TODO: panic message");

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') | KeyCode::Left => {
                    if app.column_index > 0 { app.column_index -= 1; }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if app.column_index < 2 { app.column_index += 1; }
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
        .split(f.size());

    // --- Header ---
    let header = Paragraph::new(" myeon | h/l to move • q to quit ")
        .style(Style::default().fg(FG_MUTED))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(FG_MUTED)));
    f.render_widget(header, chunks[0]);

    // --- Columns ---
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ].as_ref())
        .split(chunks[1]);

    render_column(f, columns[0], "To Do", &app.todo, app.column_index == 0);
    render_column(f, columns[1], "Doing", &app.doing, app.column_index == 1);
    render_column(f, columns[2], "Done", &app.done, app.column_index == 2);
}

fn render_column(f: &mut Frame, area: Rect, title: &str, items: &[String], is_active: bool) {
    let border_color = if is_active { BORDER_ACTIVE } else { BORDER_QUIET };

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(format!(" • {}", i)).style(Style::default().fg(FG_PRIMARY)))
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", title))
                .border_style(Style::default().fg(border_color))
        );

    f.render_widget(list, area);
}
