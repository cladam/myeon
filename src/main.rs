use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use myeon::cli;
use myeon::cli::{Cli, Commands};
use myeon::colours;
use myeon::data::{MyeonData, Priority, Task, TaskStatus};
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::BorderType;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
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

pub enum InputMode {
    Normal,
    Editing,
}

pub enum EditField {
    Title,
    Description,
    Context,
    Priority,
}

struct App {
    column_index: usize,
    selected_task_index: usize,
    all_tasks: Vec<Task>,
    current_context: String, // For filtering (e.g., "All" or "Work")
    input: String,
    input_mode: InputMode,
    is_editing_existing: bool,
    editing_task_id: Option<uuid::Uuid>,
    active_edit_field: EditField,
    editing_priority: Priority,
    editing_context: String,
    editing_description: String,
    context_list_index: usize,
}

impl App {
    fn new() -> App {
        let data = MyeonData::load();

        // If file is empty, you could optionally inject a "Welcome" task
        let tasks = if data.tasks.is_empty() {
            vec![Task {
                id: uuid::Uuid::new_v4(),
                title: "Welcome to myeon. Press 'a' to add a task.".to_string(),
                description: None,
                status: TaskStatus::Idea,
                priority: Priority::Low,
                context: "General".to_string(),
                created_at: chrono::Utc::now(),
            }]
        } else {
            data.tasks
        };

        App {
            column_index: 0,
            selected_task_index: 0,
            all_tasks: tasks,
            current_context: "All".to_string(),
            input: String::new(),
            input_mode: InputMode::Normal,
            is_editing_existing: false,
            editing_task_id: None,
            active_edit_field: EditField::Title,
            editing_priority: Priority::Low,
            editing_context: String::new(),
            editing_description: String::new(),
            context_list_index: 0,
        }
    }

    fn get_filter_contexts(&self) -> Vec<String> {
        let mut contexts = self.get_task_contexts();
        contexts.insert(0, "All".to_string());
        contexts
    }

    fn get_task_contexts(&self) -> Vec<String> {
        let mut contexts: Vec<String> = self.all_tasks.iter().map(|t| t.context.clone()).collect();
        contexts.sort();
        contexts.dedup();
        if !contexts.contains(&"General".to_string()) {
            contexts.insert(0, "General".to_string());
        }
        contexts
    }

    fn cycle_context(&mut self) {
        let available = self.get_filter_contexts();
        let current_pos = available
            .iter()
            .position(|c| c == &self.current_context)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % available.len();
        self.current_context = available[next_pos].clone();
        self.selected_task_index = 0;
    }

    fn submit_task(&mut self) {
        if self.input.is_empty() {
            return;
        }

        if self.is_editing_existing {
            if let Some(id) = self.editing_task_id {
                if let Some(task) = self.all_tasks.iter_mut().find(|t| t.id == id) {
                    task.title = self.input.clone();
                    task.description = if self.editing_description.is_empty() {
                        task.description.clone()
                    } else {
                        Some(self.editing_description.clone())
                    };
                    task.context = if self.editing_context.is_empty() {
                        task.context.clone()
                    } else {
                        self.editing_context.clone()
                    };
                    task.priority = self.editing_priority.clone();
                }
            }
            self.is_editing_existing = false;
            self.editing_task_id = None;
        } else {
            let new_task = Task {
                id: uuid::Uuid::new_v4(),
                title: self.input.clone(),
                description: None,
                status: TaskStatus::Idea,
                priority: self.editing_priority.clone(),
                context: if self.editing_context.is_empty() {
                    "General".to_string()
                } else {
                    self.editing_context.clone()
                },
                created_at: chrono::Utc::now(),
            };
            self.all_tasks.push(new_task);
        }

        // Reset all editing state
        self.input.clear();
        self.editing_context.clear();
        self.editing_priority = Priority::Low;
        self.active_edit_field = EditField::Title;
        self.context_list_index = 0;
        self.input_mode = InputMode::Normal;
        self.persist();
    }

    fn delete_task(&mut self) {
        let current_tasks = self.get_current_column_tasks();
        if let Some(task_to_delete) = current_tasks.get(self.selected_task_index) {
            let id = task_to_delete.id;
            self.all_tasks.retain(|t| t.id != id);

            // Adjust selection so it doesn't go out of bounds
            if self.selected_task_index > 0 {
                self.selected_task_index -= 1;
            }
        }
        self.persist(); //
    }

    fn start_edit(&mut self) {
        let current_tasks = self.get_current_column_tasks();
        if let Some(task) = current_tasks.get(self.selected_task_index) {
            let title = task.title.clone();
            let id = task.id;
            let context = task.context.clone();
            let priority = task.priority.clone();
            let description = task.description.clone();
            self.input = title;
            self.editing_context = context;
            self.editing_priority = priority;
            self.editing_description = description.unwrap_or_default();
            self.input_mode = InputMode::Editing;
            self.is_editing_existing = true;
            self.editing_task_id = Some(id);
        }
    }

    // Helper to get current tasks in the active column
    fn get_current_column_tasks(&self) -> Vec<&Task> {
        let status = match self.column_index {
            0 => TaskStatus::Idea,
            1 => TaskStatus::Todo,
            2 => TaskStatus::Doing,
            _ => TaskStatus::Done,
        };
        self.tasks_by_status(status)
    }

    // Move task to the next logical state
    fn move_task_forward(&mut self) {
        let current_tasks = self.get_current_column_tasks();
        if let Some(task_to_move) = current_tasks.get(self.selected_task_index) {
            let id = task_to_move.id;
            if let Some(task) = self.all_tasks.iter_mut().find(|t| t.id == id) {
                task.status = match task.status {
                    TaskStatus::Idea => TaskStatus::Todo,
                    TaskStatus::Todo => TaskStatus::Doing,
                    TaskStatus::Doing => TaskStatus::Done,
                    TaskStatus::Done => TaskStatus::Done,
                };
            }
        }
        self.persist(); //
    }

    /// Save current state back to disk
    fn persist(&self) {
        let data = MyeonData {
            tasks: self.all_tasks.clone(),
        };
        let _ = data.save();
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
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('h') | KeyCode::Left => {
                        if app.column_index > 0 {
                            app.column_index -= 1;
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if app.column_index < 3 {
                            // Changed from 2 to 3
                            app.column_index += 1;
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let max = app.get_current_column_tasks().len();
                        if app.selected_task_index + 1 < max {
                            app.selected_task_index += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.selected_task_index > 0 {
                            app.selected_task_index -= 1;
                        }
                    }
                    KeyCode::Enter => app.move_task_forward(),
                    KeyCode::Char('c') => app.cycle_context(),
                    KeyCode::Char('d') => app.delete_task(),
                    KeyCode::Char('e') => app.start_edit(),
                    _ => {}
                },
                InputMode::Editing => handle_editing_key(key, &mut app),
            }
        }
    }
}

fn handle_editing_key(key: event::KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Tab => match app.active_edit_field {
            EditField::Title => app.active_edit_field = EditField::Description,
            EditField::Description => app.active_edit_field = EditField::Context,
            EditField::Context => app.active_edit_field = EditField::Priority,
            EditField::Priority => app.active_edit_field = EditField::Title,
        },
        KeyCode::BackTab => match app.active_edit_field {
            EditField::Title => app.active_edit_field = EditField::Priority,
            EditField::Description => app.active_edit_field = EditField::Title,
            EditField::Context => app.active_edit_field = EditField::Description,
            EditField::Priority => app.active_edit_field = EditField::Context,
        },
        KeyCode::Up | KeyCode::Down if matches!(app.active_edit_field, EditField::Context) => {
            let contexts = app.get_task_contexts();
            if !contexts.is_empty() {
                if key.code == KeyCode::Down {
                    app.context_list_index = (app.context_list_index + 1) % contexts.len();
                } else if app.context_list_index > 0 {
                    app.context_list_index -= 1;
                } else {
                    app.context_list_index = contexts.len() - 1;
                }
                app.editing_context = contexts[app.context_list_index].clone();
            }
        }
        KeyCode::Enter => app.submit_task(),
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.active_edit_field = EditField::Title;
            app.input.clear();
            app.editing_description.clear();
            app.editing_context.clear();
            app.editing_priority = Priority::Low;
            app.context_list_index = 0;
        }
        KeyCode::Char(c) => match app.active_edit_field {
            EditField::Title => app.input.push(c),
            EditField::Description => app.editing_description.push(c),
            EditField::Context => app.editing_context.push(c),
            EditField::Priority => match c {
                '1' => app.editing_priority = Priority::Low,
                '2' => app.editing_priority = Priority::Medium,
                '3' => app.editing_priority = Priority::High,
                _ => {}
            },
        },
        KeyCode::Backspace => match app.active_edit_field {
            EditField::Title => {
                app.input.pop();
            }
            EditField::Description => {
                app.editing_description.pop();
            }
            EditField::Context => {
                app.editing_context.pop();
            }
            EditField::Priority => {}
        },
        _ => {}
    }
}

fn ui(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if matches!(app.input_mode, InputMode::Editing) {
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref()
        } else {
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(0),
            ]
            .as_ref()
        })
        .split(f.area());

    // --- Dynamic Header ---
    let header_text = match app.input_mode {
        InputMode::Normal => format!(
            " myeon | Context: [{}] | 'c' to cycle • 'a' to add • 'e' to edit • Enter to move card forward • 'q' to quit ",
            app.current_context.to_uppercase().to_string()
        ),
        InputMode::Editing => " Adding Task (Tab to switch fields, Enter to submit) ".to_string(),
    };

    let header_style = if let InputMode::Editing = app.input_mode {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };

    let header = Paragraph::new(header_text).style(header_style).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(header_style),
    );

    f.render_widget(header, main_chunks[0]);

    // --- Columns ---
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Idea (The Landing Strip)
            Constraint::Percentage(25), // To Do
            Constraint::Percentage(25), // Doing
            Constraint::Percentage(25), // Done
        ])
        .split(main_chunks[1]);

    let idea_tasks = app.tasks_by_status(TaskStatus::Idea);
    let todo_tasks = app.tasks_by_status(TaskStatus::Todo);
    let doing_tasks = app.tasks_by_status(TaskStatus::Doing);
    let done_tasks = app.tasks_by_status(TaskStatus::Done);

    // Determine Doing column border colour based on WIP Limit
    let doing_border_color = if doing_tasks.len() > 3 {
        ACCENT_URGENT // MutedRed indicating overwhelm
    } else if app.column_index == 2 {
        BORDER_ACTIVE // MutedTeal for focus
    } else {
        BORDER_QUIET // Dark border
    };

    render_column(
        f,
        columns[0],
        "Ideas",
        &idea_tasks,
        app.column_index == 0,
        app.selected_task_index,
        None,
    );
    render_column(
        f,
        columns[1],
        "To Do",
        &todo_tasks,
        app.column_index == 1,
        app.selected_task_index,
        None,
    );
    render_column(
        f,
        columns[2],
        "Doing",
        &doing_tasks,
        app.column_index == 2,
        app.selected_task_index,
        Some(doing_border_color),
    );
    render_column(
        f,
        columns[3],
        "Done",
        &done_tasks,
        app.column_index == 3,
        app.selected_task_index,
        None,
    );

    // --- Input Area (only in Editing mode) ---
    if matches!(app.input_mode, InputMode::Editing) {
        render_input_area(f, app, main_chunks[2]);
    }
}

fn render_column(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[&Task],
    is_active: bool,
    selected_index: usize,
    override_color: Option<Color>,
) {
    let border_color = override_color.unwrap_or(if is_active {
        BORDER_ACTIVE
    } else {
        BORDER_QUIET
    });

    // Outer column block
    let column_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(border_color));

    let inner_area = column_block.inner(area);
    f.render_widget(column_block, area);

    // Calculate card heights (3 lines per card + 1 spacing)
    let card_height = 4u16;
    let mut y_offset = 0u16;

    for (i, task) in items.iter().enumerate() {
        if y_offset + card_height > inner_area.height {
            break; // No more space
        }

        let card_area = Rect {
            x: inner_area.x,
            y: inner_area.y + y_offset,
            width: inner_area.width,
            height: card_height - 1,
        };

        let is_selected = is_active && i == selected_index;

        let priority_indicator = match task.priority {
            Priority::High => ("▌", ACCENT_URGENT),
            Priority::Medium => ("▌", Color::Rgb(192, 138, 62)),
            Priority::Low => ("▌", FG_MUTED),
        };

        let card_border_color = if is_selected {
            BORDER_ACTIVE
        } else {
            BORDER_QUIET
        };
        let card_bg = BG_DEEP;

        let description = task.description.clone().unwrap_or_default();
        let desc_preview: String = description.chars().take(30).collect();

        let card = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    priority_indicator.0,
                    Style::default().fg(priority_indicator.1),
                ),
                Span::styled(&task.title, Style::default().fg(FG_PRIMARY)),
            ]),
            Line::from(Span::styled(
                format!(" {}", desc_preview),
                Style::default().fg(FG_MUTED),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(card_border_color))
                .style(Style::default().bg(card_bg)),
        );

        f.render_widget(card, card_area);
        y_offset += card_height;
    }
}

fn render_input_area(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    // Title
    let title_style = if matches!(app.active_edit_field, EditField::Title) {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };
    let title_input = Paragraph::new(app.input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Title ")
            .border_style(title_style),
    );
    f.render_widget(title_input, chunks[0]);

    // Description
    let desc_style = if matches!(app.active_edit_field, EditField::Description) {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };
    let desc_input = Paragraph::new(app.editing_description.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Description ")
            .border_style(desc_style),
    );
    f.render_widget(desc_input, chunks[1]);

    // Context
    let context_style = if matches!(app.active_edit_field, EditField::Context) {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };
    let context_display = if app.editing_context.is_empty() {
        "↑↓ select".to_string()
    } else {
        app.editing_context.clone()
    };
    let context_input = Paragraph::new(context_display).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Context ")
            .border_style(context_style),
    );
    f.render_widget(context_input, chunks[2]);

    // Priority
    let priority_style = if matches!(app.active_edit_field, EditField::Priority) {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };
    let priority_input = Paragraph::new(format!("{:?}", app.editing_priority)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Priority ")
            .border_style(priority_style),
    );
    f.render_widget(priority_input, chunks[3]);

    if matches!(app.active_edit_field, EditField::Context) {
        render_context_popup(f, app, chunks[2]);
    }
}

fn render_context_popup(f: &mut Frame, app: &App, anchor: Rect) {
    let contexts = app.get_task_contexts();
    if contexts.is_empty() {
        return;
    }

    let popup_height = (contexts.len() as u16 + 2).min(8);
    let popup_area = Rect {
        x: anchor.x,
        y: anchor.y.saturating_sub(popup_height),
        width: anchor.width,
        height: popup_height,
    };

    let items: Vec<ListItem> = contexts
        .iter()
        .enumerate()
        .map(|(i, ctx)| {
            let style = if i == app.context_list_index {
                Style::default().fg(Color::Black).bg(BORDER_ACTIVE)
            } else {
                Style::default().fg(FG_PRIMARY)
            };
            ListItem::new(format!(" {}", ctx)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Contexts ")
            .border_style(Style::default().fg(BORDER_ACTIVE))
            .style(Style::default().bg(BG_DEEP)),
    );

    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(list, popup_area);
}
