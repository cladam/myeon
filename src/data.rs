use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Priority {
    High,   // Will use ACCENT_URGENT (MutedRed)
    Medium, // Will use QuietAmber
    Low,    // Will use MutedDetail
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub context: String, // e.g., "Work", "Personal"
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyeonData {
    pub tasks: Vec<Task>,
}
