use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "tool", content = "args")]
pub enum ToolCall {
    #[serde(rename = "read_file")]
    ReadFile { path: String },
    #[serde(rename = "write_file")]
    WriteFile { path: String, content: String },
    #[serde(rename = "run_shell")]
    RunShell { command: String },
}

pub struct ToolResult {
    pub tool_name: String,
    pub output: String,
    pub success: bool,
}

pub fn execute_tool(call: ToolCall) -> ToolResult {
    match call {
        ToolCall::ReadFile { path } => {
            match fs::read_to_string(&path) {
                Ok(content) => ToolResult {
                    tool_name: "read_file".to_string(),
                    output: content,
                    success: true,
                },
                Err(e) => ToolResult {
                    tool_name: "read_file".to_string(),
                    output: format!("Error reading file: {}", e),
                    success: false,
                },
            }
        }
        ToolCall::WriteFile { path, content } => {
            match fs::write(&path, content) {
                Ok(_) => ToolResult {
                    tool_name: "write_file".to_string(),
                    output: format!("Successfully wrote to {}", path),
                    success: true,
                },
                Err(e) => ToolResult {
                    tool_name: "write_file".to_string(),
                    output: format!("Error writing file: {}", e),
                    success: false,
                },
            }
        }
        ToolCall::RunShell { command } => {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").args(["/C", &command]).output()
            } else {
                Command::new("sh").args(["-c", &command]).output()
            };

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let combined = format!("{}{}", stdout, stderr);
                    ToolResult {
                        tool_name: "run_shell".to_string(),
                        output: if combined.is_empty() { "Success (no output)".to_string() } else { combined.to_string() },
                        success: out.status.success(),
                    }
                }
                Err(e) => ToolResult {
                    tool_name: "run_shell".to_string(),
                    output: format!("Failed to execute command: {}", e),
                    success: false,
                },
            }
        }
    }
}
