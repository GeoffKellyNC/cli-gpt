#![allow(dead_code)]
extern crate termion;

use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::exit;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

enum AiRole {
    System,
    User,
}

impl AiRole {
    fn get_string(role: &AiRole) -> String {
        match role {
            AiRole::System => String::from("system"),
            AiRole::User => String::from("user"),
        }
    }
}

struct AiReturnMessage {
    role: String,
    content: String,
}

struct Choice {
    index: u64,
    message: AiReturnMessage,
}

struct GptResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

struct AiContext {
    role: String,
    content: String,
}

fn call_gpt(user_prompt: String) -> AiReturnMessage {
    println!("{}", user_prompt);

    AiReturnMessage {
        role: AiRole::get_string(&AiRole::User),
        content: user_prompt,
    }
}

fn run_hotkey_thread(exit_flag: Arc<Mutex<bool>>, transmit: mpsc::Sender<String>) {
    println!("Running Hotkey Thread"); // DEBUG
    let ef: Arc<Mutex<bool>> = exit_flag.clone();

    let handle = thread::spawn(move || {
        let stdin = io::stdin();
        println!("Hotkey thread started"); // Debugging statement

        for c in stdin.keys() {
            println!("Key captured: {:?}", c); // Debugging statement
            let mut exit = ef.lock().unwrap();

            match c.unwrap() {
                Key::Ctrl('e') => {
                    println!("Ctrl+e pressed"); // Debugging statement
                    *exit = true;
                    break;
                }
                Key::F(8) => {
                    println!("F8 pressed"); // Debugging statement
                    transmit.send(String::from("You Pressed F8")).unwrap();
                }
                _ => {}
            }
        }
    });
}
fn main() {
    let exit_flag: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let (transmit, recieve) = mpsc::channel();

    run_hotkey_thread(exit_flag.clone(), transmit);

    println!("Welcome to GPT-CLI TOOL!");

    let ai_prompt: String = String::from("You are an AI assitant here to answer questions.");

    let context: Vec<HashMap<String, String>> = vec![{
        let mut map = HashMap::new();
        map.insert(String::from("role"), AiRole::get_string(&AiRole::System));
        map.insert(String::from("content"), ai_prompt);
        map
    }];

    loop {
        {
            let exit = exit_flag.lock().unwrap();
            if *exit {
                println!("Exiting...");
                break;
            }
        }

        if let Ok(message) = recieve.try_recv() {
            println!("Received: {}", message);
        }

        print!("Prompt ->  ");
        io::stdout().flush().unwrap();

        let mut user_prompt: String = String::new();
        std::io::stdin()
            .read_line(&mut user_prompt)
            .expect("Command Failed");

        println!("You Said: {}", user_prompt);
        io::stdout().flush().unwrap();
    }
}
