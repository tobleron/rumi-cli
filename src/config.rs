use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RumiConfig {
    pub llm: LlmConfig,
    pub prompts: PromptConfig,
    pub ui: UiConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmConfig {
    pub api_url: String,
    pub model_name: String,
    pub base_temp: f32,
    pub max_temp: f32,
    pub context_window: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptConfig {
    pub system_prompt: String,
    pub codebase_map_template: String,
    pub intent_analysis_prompt: String,
    pub summary_prompt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    pub color_primary: String,
    pub color_bold: String,
    pub color_reset: String,
}

impl RumiConfig {
    pub fn load() -> Self {
        if Path::new("rumi_config.json").exists() {
            let content = fs::read_to_string("rumi_config.json").expect("Failed to read config");
            serde_json::from_str(&content).expect("Failed to parse config")
        } else {
            let default = Self::default();
            let json = serde_json::to_string_pretty(&default).unwrap();
            fs::write("rumi_config.json", json).expect("Failed to write default config");
            default
        }
    }

    pub fn default() -> Self {
        RumiConfig {
            llm: LlmConfig {
                api_url: "http://localhost:8000/v1".to_string(),
                model_name: "qwen3-4b".to_string(),
                base_temp: 0.7,
                max_temp: 1.2,
                context_window: 24000,
            },
            prompts: PromptConfig {
                system_prompt: r###"You are Rumi, a high-context coding agent. You operate in a Think -> Act -> Observe loop. # RESPONSE FORMAT You must ALWAYS output a valid JSON object. Do not output raw text. Your output must follow one of these two structures: 1. ACTION (Use this to use a tool): {{ "type": "action", "tool": "write_file", "args": {{ "path": "test.txt", "content": "hello" }} }} 2. RESPONSE (Use this to answer the user directly when no tool is needed): {{ "type": "response", "content": "The entry point is src/Main.res..." }} # RULES 1. If a Codebase Map is provided in the context, rely on it for file paths. 2. If you need to edit a file, read it first."###.to_string(),
                
                codebase_map_template: r###"# CODEBASE MAP The following is the authoritative map of the project. ONLY use file paths found in this map. {MAP_CONTENT}"###.to_string(),

                intent_analysis_prompt: "You are a Senior Project Manager. Your job is to classify the user's intent.\n\n                Categories:\n                - 'INFO_GENERAL': General knowledge, history, greetings, or questions unrelated to the current code.\n                - 'INFO_CODE': Questions about this specific codebase, architecture, or files.\n                - 'ACTION': Requests to modify code, create files, run commands, or plan features.\n                \n                Reply with ONLY one of the three tags. No other text.".to_string(),
                
                summary_prompt: "Summarize the following conversation history into a concise context block. Retain key decisions, facts, file paths, and code snippets.".to_string(),
            },
            ui: UiConfig {
                color_primary: "\x1b[38;5;166m".to_string(),
                color_bold: "\x1b[1;38;5;166m".to_string(),
                color_reset: "\x1b[0m".to_string(),
            },
        }
    }
}