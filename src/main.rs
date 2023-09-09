#![allow(dead_code)]
extern crate termion;

use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::Write;

const GPT_MODEL: &str = "gpt-3.5-turbo";
enum AiRole {
    System,
    Assistant,
    User,
}

impl AiRole {
    fn get_string(role: &AiRole) -> String {
        match role {
            AiRole::System => String::from("system"),
            AiRole::User => String::from("user"),
            AiRole::Assistant => String::from("assistant"),
        }
    }
}

#[derive(Clone)]
struct ContextContent {
    data: Vec<HashMap<String, String>>,
}

impl ContextContent {
    fn new() -> Self {
        ContextContent { data: Vec::new() }
    }

    fn add_to_context(&mut self, role: String, text: &str) {
        let mut context_content_map: HashMap<String, String> = HashMap::new();

        if role == AiRole::get_string(&AiRole::System) {
            context_content_map.insert(String::from("role"), AiRole::get_string(&AiRole::System));
        } else if role == AiRole::get_string(&AiRole::User) {
            context_content_map.insert(String::from("role"), AiRole::get_string(&AiRole::User));
        } else if role == AiRole::get_string(&AiRole::Assistant) {
            context_content_map
                .insert(String::from("role"), AiRole::get_string(&AiRole::Assistant));
        }

        context_content_map.insert(String::from("content"), String::from(text));

        self.data.push(context_content_map);
    }
}
#[derive(Deserialize, Debug)]
struct Choice {
    index: i64,
    message: AiContext,
    finish_reason: String,
}
#[derive(Deserialize, Debug)]
struct AiUsage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}
#[derive(Deserialize, Debug)]
struct GptResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: AiUsage,
}
#[derive(Deserialize, Debug)]
struct AiContext {
    role: String,
    content: String,
}

async fn call_gpt(
    context_vec: ContextContent,
) -> Result<GptResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => panic!("OPENAI_API_KEY not set"),
    };

    let payload = json!({
        "model": GPT_MODEL,
        "messages": context_vec.data
    });

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let raw = res.text().await?;

        // Deserialize the response
        let ai_res: GptResponse = serde_json::from_str(&raw)?;
        Ok(ai_res)
    } else {
        let err_msg = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("No additional error message"));

        println!("Error: {}", err_msg);

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "HTTP Error",
        )))
    }
}

fn init_context_vec() -> ContextContent {
    let mut context_vec: ContextContent = ContextContent::new();

    let ai_init_prompt: &str = "You Area a personal AI assistant.";

    context_vec.add_to_context(AiRole::get_string(&AiRole::System), ai_init_prompt);

    context_vec
}
#[tokio::main]
async fn main() {
    let mut gpt_context: ContextContent = init_context_vec();

    loop {
        let mut user_input: String = String::new();

        print!("Input -> ");
        std::io::stdout().flush().unwrap();

        std::io::stdin().read_line(&mut user_input).unwrap();

        gpt_context.add_to_context(AiRole::get_string(&AiRole::User), &user_input);

        match call_gpt(gpt_context.clone()).await {
            Ok(response) => {
                if let Some(ai_choices) = response.choices.get(0) {
                    let ai_response_text: &str = &ai_choices.message.content;

                    println!("GPT => {}", ai_response_text);
                    continue;
                }
            }
            Err(e) => {
                println!("An error occured {:?}", e);
            }
        }

        continue;
    }
}
