use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::llm_client::ChatMessage;

pub struct SessionManager {
    pub session_id: String,
    pub session_path: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let id = format!("SID_{}", since_the_epoch.as_secs());
        
        let base_path = Path::new("sessions");
        let session_path = base_path.join(&id);
        fs::create_dir_all(&session_path).expect("Failed to create session directory");

        SessionManager {
            session_id: id,
            session_path,
        }
    }

    pub fn save_history(&self, history: &Vec<ChatMessage>) {
        let file_path = self.session_path.join("history.json");
        let json = serde_json::to_string_pretty(history).unwrap_or_default();
        let _ = fs::write(file_path, json);
    }
}
