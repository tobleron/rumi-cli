use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Complexity {
    Simple,
    Complex,
}

pub struct ThinkingEngine;

impl ThinkingEngine {
    /// Generates a unique ID for the thought process (e.g., TP_1712345678)
    fn generate_tp_id() -> String {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        format!("TP_{}", since_the_epoch.as_secs())
    }

    /// Initializes a new Thought Process environment
    pub fn init_thought_process(query: &str, context_map: &str) -> std::io::Result<String> {
        let tp_id = Self::generate_tp_id();
        let base_path = Path::new("thinking");
        let tp_path = base_path.join(&tp_id);

        // 1. Create directory
        fs::create_dir_all(&tp_path)?;

        // 2. Create input.md
        let input_file = tp_path.join("input.md");
        let content = format!(
            "# Thought Process: {}

## Original Query
{}

## Context Map
{}

## Instructions
Analyze the request and create a step-by-step plan in `plan.md`.",
            tp_id, query, context_map
        );
        fs::write(input_file, content)?;

        Ok(tp_id)
    }
}
