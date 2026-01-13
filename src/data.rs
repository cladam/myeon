use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

impl MyeonData {
    /// Gets the platform-specific config directory:
    /// e.g., ~/.config/myeon/tasks.json on Linux
    fn get_data_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("com", "ilseon", "myeon")
            .expect("Could not determine config directory");

        let mut path = proj_dirs.config_dir().to_path_buf();
        // Ensure directory exists
        let _ = fs::create_dir_all(&path);
        path.push("tasks.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::get_data_path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or(MyeonData { tasks: vec![] })
        } else {
            MyeonData { tasks: vec![] }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_data_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
