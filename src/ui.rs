use crate::app::{App, EditField, InputMode};
use crate::data::{Priority, Task, TaskStatus};
use ratatui::style::Modifier;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

pub const BG_DEEP: Color = Color::Rgb(54, 52, 58);
pub const FG_PRIMARY: Color = Color::Rgb(224, 224, 224);
pub const FG_MUTED: Color = Color::Rgb(176, 176, 176);
pub const BORDER_ACTIVE: Color = Color::Rgb(90, 155, 128);
pub const BORDER_QUIET: Color = Color::Rgb(31, 31, 31);
pub const ACCENT_URGENT: Color = Color::Rgb(179, 95, 95);

pub fn render(f: &mut Frame, app: &App) {
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

    render_header(f, app, main_chunks[0]);
    render_columns(f, app, main_chunks[1]);

    if matches!(app.input_mode, InputMode::Editing) {
        render_input_area(f, app, main_chunks[2]);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header_text = match app.input_mode {
        InputMode::Normal => format!(" myeon | Context: [{}]", app.current_context.to_uppercase()),
        InputMode::Editing => " Adding Task (Tab to switch fields, Enter to submit) ".to_string(),
    };

    let header_style = if matches!(app.input_mode, InputMode::Editing) {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };

    let header = Paragraph::new(header_text).style(header_style).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(header_style),
    );
    f.render_widget(header, area);
}

fn render_columns(f: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let doing_tasks = app.tasks_by_status(TaskStatus::Doing);
    let doing_border_color = if doing_tasks.len() > 3 {
        ACCENT_URGENT
    } else if app.column_index == 2 {
        BORDER_ACTIVE
    } else {
        BORDER_QUIET
    };

    render_column(
        f,
        columns[0],
        "Ideas",
        &app.tasks_by_status(TaskStatus::Idea),
        app.column_index == 0,
        app.selected_task_index,
        None,
        app.column_index != 0, // is_dimmed
    );
    render_column(
        f,
        columns[1],
        "To Do",
        &app.tasks_by_status(TaskStatus::Todo),
        app.column_index == 1,
        app.selected_task_index,
        None,
        app.column_index != 1, // is_dimmed
    );
    render_column(
        f,
        columns[2],
        "Doing",
        &doing_tasks,
        app.column_index == 2,
        app.selected_task_index,
        Some(doing_border_color),
        app.column_index != 2, // is_dimmed
    );
    render_column(
        f,
        columns[3],
        "Done",
        &app.tasks_by_status(TaskStatus::Done),
        app.column_index == 3,
        app.selected_task_index,
        None,
        app.column_index != 3, // is_dimmed
    );
}

fn render_column(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[&Task],
    is_active: bool,
    selected_index: usize,
    override_color: Option<Color>,
    is_dimmed: bool,
) {
    let border_color = override_color.unwrap_or(if is_active {
        BORDER_ACTIVE
    } else {
        BORDER_QUIET
    });

    // Dim factor for inactive columns
    let dim_style = if is_dimmed {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default()
    };

    let fg_primary = if is_dimmed {
        Color::Rgb(80, 80, 80)
    } else {
        FG_PRIMARY
    };
    let fg_muted = if is_dimmed {
        Color::Rgb(50, 50, 50)
    } else {
        FG_MUTED
    };

    let column_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(border_color).patch(dim_style));

    let inner_area = column_block.inner(area);
    f.render_widget(column_block, area);

    let mut y_offset = 0u16;

    for (i, task) in items.iter().enumerate() {
        let content_width = inner_area.width.saturating_sub(4) as usize;
        let wrapped_title = wrap_text(&task.title, content_width.saturating_sub(1));
        let description = task.description.clone().unwrap_or_default();
        let wrapped_desc = wrap_text(&description, content_width);

        // Calculate height: borders (2) + title lines + description lines
        let title_lines = wrapped_title.lines().count().max(1) as u16;
        let desc_lines = if wrapped_desc.is_empty() {
            0
        } else {
            wrapped_desc.lines().count() as u16
        };
        let card_height = 2 + title_lines + desc_lines; // 2 for top/bottom border

        if y_offset + card_height > inner_area.height {
            break;
        }

        let card_area = Rect {
            x: inner_area.x,
            y: inner_area.y + y_offset,
            width: inner_area.width,
            height: card_height,
        };
        let is_selected = is_active && i == selected_index;

        let (indicator, indicator_color) = match task.priority {
            Priority::High => (
                "▌",
                if is_dimmed {
                    Color::Rgb(60, 40, 40)
                } else {
                    ACCENT_URGENT
                },
            ),
            Priority::Medium => (
                "▌",
                if is_dimmed {
                    Color::Rgb(60, 50, 30)
                } else {
                    Color::Rgb(192, 138, 62)
                },
            ),
            Priority::Low => ("▌", fg_muted),
        };

        let card_border_color = if is_selected {
            BORDER_ACTIVE
        } else if is_dimmed {
            Color::Rgb(40, 40, 40)
        } else {
            BORDER_QUIET
        };

        let mut lines: Vec<Line> = wrapped_title
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                if idx == 0 {
                    Line::from(vec![
                        Span::styled(indicator, Style::default().fg(indicator_color)),
                        Span::styled(line.to_string(), Style::default().fg(fg_primary)),
                    ])
                } else {
                    Line::from(Span::styled(
                        format!(" {}", line),
                        Style::default().fg(fg_primary),
                    ))
                }
            })
            .collect();

        if !wrapped_desc.is_empty() {
            for line in wrapped_desc.lines() {
                lines.push(Line::from(Span::styled(
                    format!(" {}", line),
                    Style::default().fg(fg_muted),
                )));
            }
        }

        let card = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(card_border_color))
                .style(Style::default().bg(BG_DEEP)),
        );

        f.render_widget(card, card_area);
        y_offset += card_height + 1; // +1 for spacing between cards
    }
}

fn wrap_text(text: &str, max_width: usize) -> String {
    if max_width == 0 || text.is_empty() {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line_len = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if current_line_len + word_len + 1 > max_width && current_line_len > 0 {
            result.push('\n');
            current_line_len = 0;
        }

        if current_line_len > 0 {
            result.push(' ');
            current_line_len += 1;
        }

        result.push_str(word);
        current_line_len += word_len;
    }

    result
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

    render_input_field(
        f,
        " Title ",
        &app.input,
        matches!(app.active_edit_field, EditField::Title),
        chunks[0],
    );
    render_input_field(
        f,
        " Description ",
        &app.editing_description,
        matches!(app.active_edit_field, EditField::Description),
        chunks[1],
    );

    let context_display = if app.editing_context.is_empty() {
        "↑↓ select".to_string()
    } else {
        app.editing_context.clone()
    };
    render_input_field(
        f,
        " Context ",
        &context_display,
        matches!(app.active_edit_field, EditField::Context),
        chunks[2],
    );
    render_input_field(
        f,
        " Priority ",
        &format!("{:?}", app.editing_priority),
        matches!(app.active_edit_field, EditField::Priority),
        chunks[3],
    );

    if matches!(app.active_edit_field, EditField::Context) {
        render_context_popup(f, app, chunks[2]);
    }
}

fn render_input_field(f: &mut Frame, title: &str, content: &str, is_active: bool, area: Rect) {
    let style = if is_active {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(FG_MUTED)
    };
    let input = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(style),
    );
    f.render_widget(input, area);
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
