use crate::data::{MyeonData, Priority, Task, TaskStatus};

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

pub struct App {
    pub column_index: usize,
    pub selected_task_index: usize,
    pub all_tasks: Vec<Task>,
    pub current_context: String,
    pub input: String,
    pub input_mode: InputMode,
    pub is_editing_existing: bool,
    pub editing_task_id: Option<uuid::Uuid>,
    pub active_edit_field: EditField,
    pub editing_priority: Priority,
    pub editing_context: String,
    pub editing_description: String,
    pub context_list_index: usize,
}

impl App {
    pub fn new() -> App {
        let data = MyeonData::load();
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

    pub fn get_filter_contexts(&self) -> Vec<String> {
        let mut contexts = self.get_task_contexts();
        contexts.insert(0, "All".to_string());
        contexts
    }

    pub fn get_task_contexts(&self) -> Vec<String> {
        let mut contexts: Vec<String> = self.all_tasks.iter().map(|t| t.context.clone()).collect();
        contexts.sort();
        contexts.dedup();
        if !contexts.contains(&"General".to_string()) {
            contexts.insert(0, "General".to_string());
        }
        contexts
    }

    pub fn cycle_context(&mut self) {
        let available = self.get_filter_contexts();
        let current_pos = available
            .iter()
            .position(|c| c == &self.current_context)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % available.len();
        self.current_context = available[next_pos].clone();
        self.selected_task_index = 0;
    }

    pub fn submit_task(&mut self) {
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

        self.reset_editing_state();
        self.persist();
    }

    pub fn delete_task(&mut self) {
        let current_tasks = self.get_current_column_tasks();
        if let Some(task_to_delete) = current_tasks.get(self.selected_task_index) {
            let id = task_to_delete.id;
            self.all_tasks.retain(|t| t.id != id);
            if self.selected_task_index > 0 {
                self.selected_task_index -= 1;
            }
        }
        self.persist();
    }

    pub fn start_edit(&mut self) {
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

    pub fn get_current_column_tasks(&self) -> Vec<&Task> {
        let status = match self.column_index {
            0 => TaskStatus::Idea,
            1 => TaskStatus::Todo,
            2 => TaskStatus::Doing,
            _ => TaskStatus::Done,
        };
        self.tasks_by_status(status)
    }

    pub fn move_task_forward(&mut self) {
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
        self.persist();
    }

    pub fn move_task_backward(&mut self) {
        let current_tasks = self.get_current_column_tasks();
        if let Some(task_to_move) = current_tasks.get(self.selected_task_index) {
            let id = task_to_move.id;
            if let Some(task) = self.all_tasks.iter_mut().find(|t| t.id == id) {
                task.status = match task.status {
                    TaskStatus::Idea => TaskStatus::Idea,
                    TaskStatus::Todo => TaskStatus::Idea,
                    TaskStatus::Doing => TaskStatus::Todo,
                    TaskStatus::Done => TaskStatus::Doing,
                };
            }
        }
        self.persist();
    }

    pub fn tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self
            .all_tasks
            .iter()
            .filter(|t| t.status == status)
            .filter(|t| self.current_context == "All" || t.context == self.current_context)
            .collect();

        tasks.sort_by(|a, b| {
            let priority_order = |p: &Priority| match p {
                Priority::High => 0,
                Priority::Medium => 1,
                Priority::Low => 2,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
            // Need Urgency later to use Eisenhower matrix
            // .then_with(|| a.urgency.cmp(&b.urgency))
        });

        tasks
    }

    fn persist(&self) {
        let data = MyeonData {
            tasks: self.all_tasks.clone(),
        };
        let _ = data.save();
    }

    fn reset_editing_state(&mut self) {
        self.input.clear();
        self.editing_context.clear();
        self.editing_description.clear();
        self.editing_priority = Priority::Low;
        self.active_edit_field = EditField::Title;
        self.context_list_index = 0;
        self.input_mode = InputMode::Normal;
    }
}
