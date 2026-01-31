use clap::Parser;

mod tools;
mod map_parser;
mod thinking_engine;
mod llm_client;
mod session_manager;
mod config;

use tools::{execute_tool, AgentOutput};
use map_parser::MapParser;
use thinking_engine::ThinkingEngine;
use llm_client::{LlmClient, ChatMessage};
use session_manager::SessionManager;
use config::RumiConfig;

use std::io::{self, Write};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The query to send to Rumi
    #[arg(short, long, default_value = "Analyze the map and tell me what the entry point of the application is.")]
    query: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = RumiConfig::load();
    println!("{}Rumi-CLI: Active and connected to vLLM (24k Context){}", config.ui.color_bold, config.ui.color_reset);

    let client = LlmClient::new(&config);
    let session_manager = SessionManager::new();
    println!("Session ID: {}", session_manager.session_id);

    // Load the Map
    let project_map = MapParser::get_context_map();
    println!("Loaded MAP.md ({} bytes)", project_map.len());
    println!("Type {}/exit{} or {}/quit{} to stop.", config.ui.color_primary, config.ui.color_reset, config.ui.color_primary, config.ui.color_reset);

    // Initial query from args (if provided explicitly, run it first)
    // If it's the default value, we can choose to skip it or run it. 
    // For now, let's treat the arg as the "first" command, then drop to REPL.
    
    let mut current_query = args.query.clone();

    // Check if the user actually provided a query or if it's just the default
    // This is a heuristic; technically a user could type the default string.
    let default_query = "Analyze the map and tell me what the entry point of the application is.";
    if current_query == default_query {
        println!("\n--- {}Interactive Mode{} ---", config.ui.color_bold, config.ui.color_reset);
        print!("{}>{} ", config.ui.color_bold, config.ui.color_reset);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        current_query = input.trim().to_string();
    }

    // Initialize Chat History
    let mut chat_history: Vec<ChatMessage> = vec![
        ChatMessage {
            role: "system".to_string(),
            content: config.prompts.system_prompt.clone(),
        }
    ];
    let mut map_loaded = false;

    loop {
        // Auto-save history
        session_manager.save_history(&chat_history);

        if current_query.eq_ignore_ascii_case("exit") 
           || current_query.eq_ignore_ascii_case("quit")
           || current_query.eq_ignore_ascii_case("/exit")
           || current_query.eq_ignore_ascii_case("/quit")
        {
            println!("{}Rumi-CLI: Goodbye!{}", config.ui.color_bold, config.ui.color_reset);
            break;
        }

        if current_query.eq_ignore_ascii_case("/compress") {
            if chat_history.len() <= 3 {
                println!("{}History too short to compress.{}", config.ui.color_primary, config.ui.color_reset);
            } else {
                println!("{}Compressing history...{}", config.ui.color_bold, config.ui.color_reset);
                
                // Keep System (0) and Last 2
                let system_msg = chat_history[0].clone();
                let last_two = chat_history[chat_history.len()-2..].to_vec();
                
                // Summarize the middle
                let middle_msgs = chat_history[1..chat_history.len()-2].to_vec();
                let summary_prompt_msg = ChatMessage {
                    role: "system".to_string(),
                    content: config.prompts.summary_prompt.clone(),
                };
                
                let mut summarization_context = vec![summary_prompt_msg];
                summarization_context.extend(middle_msgs);

                match client.chat_completion(summarization_context, 0, false).await {
                    Ok((summary, _)) => {
                        println!("{}Compression Complete.{}", config.ui.color_primary, config.ui.color_reset);
                        let summary_msg = ChatMessage {
                            role: "system".to_string(),
                            content: format!("*** PREVIOUS CONTEXT SUMMARY ***\n{}", summary),
                        };
                        
                        // Rebuild history
                        chat_history = vec![system_msg, summary_msg];
                        chat_history.extend(last_two);
                        session_manager.save_history(&chat_history);
                    },
                    Err(e) => println!("Compression failed: {}", e),
                }
            }
            
            // Prompt again
            print!("\n{}>{} ", config.ui.color_bold, config.ui.color_reset);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            current_query = input.trim().to_string();
            continue;
        }

        if current_query.is_empty() {
             print!("{}>{} ", config.ui.color_bold, config.ui.color_reset);
             io::stdout().flush().unwrap();
             let mut input = String::new();
             io::stdin().read_line(&mut input).expect("Failed to read line");
             current_query = input.trim().to_string();
             continue;
        }

        // --- Intent Analysis ---
        let intent_prompt = config.prompts.intent_analysis_prompt.clone();
        
        let intent_messages = vec![
            ChatMessage { role: "system".to_string(), content: intent_prompt },
            ChatMessage { role: "user".to_string(), content: current_query.clone() }
        ];

        let mut current_task_input = current_query.clone();

        match client.chat_completion(intent_messages, 0, false).await {
            Ok((c_res, _)) => {
                let classification = c_res.trim().to_uppercase();
                
                // Lazy Load Map if needed (INFO_CODE or ACTION)
                if (classification.contains("ACTION") || classification.contains("INFO_CODE")) && !map_loaded {
                    let map_msg = config.prompts.codebase_map_template.replace("{MAP_CONTENT}", &project_map);
                    chat_history.insert(1, ChatMessage {
                        role: "system".to_string(),
                        content: map_msg,
                    });
                    map_loaded = true;
                    println!("{}Context: Codebase Map loaded.{}", config.ui.color_primary, config.ui.color_reset);
                }

                if classification.contains("ACTION") {
                    match ThinkingEngine::init_thought_process(&current_query, &project_map) {
                        Ok(tp_id) => {
                            println!("\n{}🧠 Initiating Action Plan: {} 🧠{}", config.ui.color_bold, tp_id, config.ui.color_reset);
                            println!("Environment: thinking/{}/input.md", tp_id);
                            
                            // Redirect the agent to the thought process
                            current_task_input = format!(
                                "ID: {}. MANDATORY STEPS:\n\
                                1. Use 'read_file' to read 'thinking/{}/input.md'.\n\
                                2. Use 'write_file' to create 'thinking/{}/plan.md' with a detailed implementation strategy.\n\
                                3. Confirm only after BOTH tools have been called.", 
                                tp_id, tp_id, tp_id
                            );
                        },
                        Err(e) => eprintln!("Failed to init thought process: {}", e),
                    }
                }
            },
            Err(e) => eprintln!("Intent check failed: {}", e),
        }

        // Add User Query to History
        chat_history.push(ChatMessage {
            role: "user".to_string(),
            content: current_task_input,
        });

        let mut loop_count = 0;
        
        println!("{}Processing...{}", config.ui.color_primary, config.ui.color_reset);

        // Task Execution Loop
        loop {
            let start_time = Instant::now();
            match client.chat_completion(chat_history.clone(), loop_count, false).await {
                Ok((response, usage_opt)) => {
                    let duration = start_time.elapsed();
                    if let Some(usage) = usage_opt {
                        let tps = usage.completion_tokens as f64 / duration.as_secs_f64();
                        println!("{}(Speed: {:.1} tok/s | In: {} | Out: {}){}", config.ui.color_primary, tps, usage.prompt_tokens, usage.completion_tokens, config.ui.color_reset);
                    }
                    
                    // Add Assistant Response to History
                    chat_history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response.clone(),
                    });

                    if let Some(json_start) = response.find('{') {
                        if let Some(json_end) = response.rfind('}') {
                            let json_str = &response[json_start..json_end + 1];
                            
                            match serde_json::from_str::<AgentOutput>(json_str) {
                                Ok(output) => {
                                    match output {
                                        AgentOutput::Response { content } => {
                                            println!("\n{}Rumi: {}{}", config.ui.color_bold, config.ui.color_reset, content);
                                            break; // Task Complete
                                        }
                                        AgentOutput::Action { tool, args } => {
                                            println!("\n{}--- Action: {} ---{}", config.ui.color_bold, tool, config.ui.color_reset);
                                            let result = execute_tool(&tool, &args);
                                            println!("{}", result.output);
                                            
                                            // SPECIAL: Notify user when a plan is created
                                            if tool == "write_file" && args.get("path").and_then(|v| v.as_str()).map(|s| s.contains("plan.md")).unwrap_or(false) {
                                                println!("\n{}✨ ACTION PLAN CREATED: {} ✨{}", config.ui.color_bold, args.get("path").unwrap(), config.ui.color_reset);
                                            }

                                            // Add Tool Result to History (as User Observation)
                                            chat_history.push(ChatMessage {
                                                role: "user".to_string(),
                                                content: format!("Observation from {}:\n{}", result.tool_name, result.output),
                                            });

                                            loop_count += 1;
                                            
                                            if loop_count > 5 {
                                                println!("Max loops reached.");
                                                break;
                                            }
                                            continue;
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("\n--- {}Rumi (Raw/Invalid JSON){} ---\n{}", config.ui.color_primary, config.ui.color_reset, response);
                                    println!("JSON Parse Error: {}", e);
                                    break;
                                }
                            }
                        }
                    } else {
                        // No JSON found, print raw response
                        println!("\n--- {}Rumi (Text){} ---\n{}", config.ui.color_primary, config.ui.color_reset, response);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }

        // Reset for next interaction
        print!("\n{}>{} ", config.ui.color_bold, config.ui.color_reset);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        current_query = input.trim().to_string();
    }



}
