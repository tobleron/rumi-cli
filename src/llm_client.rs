use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::RumiConfig;

#[derive(Serialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    api_url: String,
    model_name: String,
    base_temp: f32,
    max_temp: f32,
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
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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
    pub fn new(config: &RumiConfig) -> Self {
        LlmClient {
            client: Client::new(),
            api_url: config.llm.api_url.clone(),
            model_name: config.llm.model_name.clone(),
            base_temp: config.llm.base_temp,
            max_temp: config.llm.max_temp,
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
        messages: Vec<ChatMessage>,
        loop_count: u32,
        is_complex: bool,
    ) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>> {
        let temp = self.calculate_temperature(loop_count, is_complex);

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
            Ok((choice.message.content.clone(), response_json.usage))
        } else {
            Err("No content in response".into())
        }
    }
}
