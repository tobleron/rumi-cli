use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AgentOutput {
    #[serde(rename = "response")]
    Response { content: String },
    
    #[serde(rename = "action")]
    Action { 
        tool: String, 
        args: Value 
    },
}

pub struct ToolResult {
    pub tool_name: String,
    pub output: String,
    #[allow(dead_code)]
    pub success: bool,
}

pub fn execute_tool(tool_name: &str, args: &Value) -> ToolResult {
    match tool_name {
        "read_file" => {
            if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                match fs::read_to_string(path) {
                    Ok(content) => ToolResult {
                        tool_name: tool_name.to_string(),
                        output: content,
                        success: true,
                    },
                    Err(e) => ToolResult {
                        tool_name: tool_name.to_string(),
                        output: format!("Error reading file: {}", e),
                        success: false,
                    },
                }
            } else {
                 ToolResult {
                    tool_name: tool_name.to_string(),
                    output: "Missing 'path' argument".to_string(),
                    success: false,
                }
            }
        }
        "write_file" => {
            let path_opt = args.get("path").and_then(|v| v.as_str());
            let content_opt = args.get("content").and_then(|v| v.as_str());
            
            if let (Some(path), Some(content)) = (path_opt, content_opt) {
                match fs::write(path, content) {
                    Ok(_) => ToolResult {
                        tool_name: tool_name.to_string(),
                        output: format!("Successfully wrote to {}", path),
                        success: true,
                    },
                    Err(e) => ToolResult {
                        tool_name: tool_name.to_string(),
                        output: format!("Error writing file: {}", e),
                        success: false,
                    },
                }
            } else {
                ToolResult {
                    tool_name: tool_name.to_string(),
                    output: "Missing 'path' or 'content' argument".to_string(),
                    success: false,
                }
            }
        }
        "run_shell" => {
            if let Some(command) = args.get("command").and_then(|v| v.as_str()) {
                let output = if cfg!(target_os = "windows") {
                    Command::new("cmd").args(["/C", command]).output()
                } else {
                    Command::new("sh").args(["-c", command]).output()
                };

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        let combined = format!("{}{}", stdout, stderr);
                        ToolResult {
                            tool_name: tool_name.to_string(),
                            output: if combined.is_empty() { "Success (no output)".to_string() } else { combined.to_string() },
                            success: out.status.success(),
                        }
                    }
                    Err(e) => ToolResult {
                        tool_name: tool_name.to_string(),
                        output: format!("Failed to execute command: {}", e),
                        success: false,
                    },
                }
            } else {
                ToolResult {
                    tool_name: tool_name.to_string(),
                    output: "Missing 'command' argument".to_string(),
                    success: false,
                }
            }
        }
        _ => ToolResult {
            tool_name: tool_name.to_string(),
            output: format!("Unknown tool: {}", tool_name),
            success: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn test_write_file() {
        let path = "test_output_v2.txt";
        let content = "Hello, V2!";
        let args = json!({ "path": path, "content": content });

        let result = execute_tool("write_file", &args);
        assert!(result.success);
        assert_eq!(result.tool_name, "write_file");

        let read_content = fs::read_to_string(path).expect("Failed to read file");
        assert_eq!(read_content, content);

        fs::remove_file(path).expect("Failed to delete test file");
    }
}
