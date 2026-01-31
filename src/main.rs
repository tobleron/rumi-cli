use dotenvy::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    api_url: String,
    model_name: String,
    base_temp: f32,
    max_temp: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct CompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize, Debug)]
struct MessageContent {
    content: String,
}

impl LlmClient {
    pub fn new() -> Self {
        dotenv().ok();
        let api_url = env::var("VLLM_API_URL").expect("VLLM_API_URL must be set");
        let model_name = env::var("MODEL_NAME").expect("MODEL_NAME must be set");
        let base_temp = env::var("BASE_TEMPERATURE")
            .unwrap_or("0.7".to_string())
            .parse()
            .unwrap_or(0.7);
        let max_temp = env::var("MAX_TEMPERATURE")
            .unwrap_or("1.2".to_string())
            .parse()
            .unwrap_or(1.2);

        LlmClient {
            client: Client::new(),
            api_url,
            model_name,
            base_temp,
            max_temp,
        }
    }

    fn calculate_temperature(&self, loop_count: u32, is_complex: bool) -> f32 {
        let start_temp = if is_complex {
            self.base_temp + 0.1
        } else {
            self.base_temp
        };
        let dynamic_temp = start_temp + (loop_count as f32 * 0.1);
        if dynamic_temp > self.max_temp {
            self.max_temp
        } else {
            dynamic_temp
        }
    }

    pub async fn chat_completion(
        &self,
        system_prompt: &str,
        user_query: &str,
        loop_count: u32,
        is_complex: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let temp = self.calculate_temperature(loop_count, is_complex);
        println!("Thinking with Temp: {}, Attempt: {}", temp, loop_count + 1);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_query.to_string(),
            },
        ];

        let request_body = CompletionRequest {
            model: self.model_name.clone(),
            messages,
            temperature: temp,
            max_tokens: 2048,
        };

        let url = format!("{}/chat/completions", self.api_url);
        
        let res = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
             let error_text = res.text().await?;
             return Err(format!("API Error: {}", error_text).into());
        }

        let response_json: CompletionResponse = res.json().await?;
        
        if let Some(choice) = response_json.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No content in response".into())
        }
    }
}

mod tools;

mod map_parser;



use tools::{execute_tool, ToolCall};

use map_parser::MapParser;



// ... existing LlmClient ...



#[tokio::main]

async fn main() {

    println!("Rumi-CLI: Active and connected to vLLM (24k Context)");

    let client = LlmClient::new();



    // Load the Map

    let project_map = MapParser::get_context_map();

    println!("Loaded MAP.md ({} bytes)", project_map.len());



    let system_prompt = format!(r#"You are Rumi, a high-context coding agent.

You operate in a Think -> Act -> Observe loop.



# CODEBASE MAP

The following is the authoritative map of the project. ONLY use file paths found in this map.

{}



# TOOL USAGE

To perform actions, you MUST output a valid JSON object:

{{

  "tool": "read_file",

  "args": {{ "path": "src/Main.res" }}

}}

OR

{{

  "tool": "write_file",

  "args": {{ "path": "path/to/file", "content": "..." }}

}}

OR

{{

  "tool": "run_shell",

  "args": {{ "command": "cargo check" }}

}}



# RULES

1. Always explain your reasoning briefly before outputting the JSON tool call.

2. Rely on the Codebase Map to find files. Do not guess paths.

3. If you need to edit a file, read it first."#, project_map);



    let mut user_query = String::from("Analyze the map and tell me what the entry point of the application is.");

    let mut loop_count = 0;



    loop {

        match client.chat_completion(&system_prompt, &user_query, loop_count, false).await {

            Ok(response) => {

                println!("\n--- Rumi Thinks ---\n{}", response);



                // Simple parser for JSON in response

                if let Some(json_start) = response.find('{') {

                    if let Some(json_end) = response.rfind('}') {

                        let json_str = &response[json_start..json_end + 1];

                        if let Ok(tool_call) = serde_json::from_str::<ToolCall>(json_str) {

                            let result = execute_tool(tool_call);

                            println!("\n--- Tool Execution ({}) ---\n{}", result.tool_name, result.output);

                            

                            // Feed the observation back into the next loop

                            user_query = format!("Observation from {}:\n{}", result.tool_name, result.output);

                            loop_count += 1;

                            

                            if loop_count > 5 {

                                println!("Max loops reached. Stopping.");

                                break;

                            }

                            continue;

                        }

                    }

                }

                

                println!("\nTask appears complete or no tool call found.");

                break;

            }

            Err(e) => {

                eprintln!("Error: {}", e);

                break;

            }

        }

    }

}


