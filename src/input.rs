use crate::app::{App, EditField, InputMode};
use crate::data::Priority;
use crossterm::event::{self, Event, KeyCode};

pub fn handle_input(app: &mut App) -> std::io::Result<bool> {
    if let Event::Key(key) = event::read()? {
        match app.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('a') => app.input_mode = InputMode::Editing,
                KeyCode::Char('h') | KeyCode::Left => {
                    if app.column_index > 0 {
                        app.column_index -= 1;
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if app.column_index < 3 {
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
                KeyCode::Backspace => app.move_task_backward(),
                KeyCode::Char('c') => app.cycle_context(),
                KeyCode::Char('d') => app.delete_task(),
                KeyCode::Char('e') => app.start_edit(),
                _ => {}
            },
            InputMode::Editing => handle_editing_key(key, app),
        }
    }
    Ok(false)
}

fn handle_editing_key(key: event::KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Tab => {
            app.active_edit_field = match app.active_edit_field {
                EditField::Title => EditField::Description,
                EditField::Description => EditField::Context,
                EditField::Context => EditField::Priority,
                EditField::Priority => EditField::Title,
            }
        }
        KeyCode::BackTab => {
            app.active_edit_field = match app.active_edit_field {
                EditField::Title => EditField::Priority,
                EditField::Description => EditField::Title,
                EditField::Context => EditField::Description,
                EditField::Priority => EditField::Context,
            }
        }
        KeyCode::Up | KeyCode::Down if matches!(app.active_edit_field, EditField::Context) => {
            let contexts = app.get_task_contexts();
            if !contexts.is_empty() {
                app.context_list_index = if key.code == KeyCode::Down {
                    (app.context_list_index + 1) % contexts.len()
                } else if app.context_list_index > 0 {
                    app.context_list_index - 1
                } else {
                    contexts.len() - 1
                };
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
