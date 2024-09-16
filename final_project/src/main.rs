use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;
use ureq;

const CACHE_LIMIT: usize = 10;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheEntry {
    prompt: String,
    response: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cache {
    entries: Vec<CacheEntry>,
}

impl Cache {
    fn add_entry(&mut self, prompt: String, response: String) {
        if self.entries.len() >= CACHE_LIMIT {
            self.entries.remove(0); // Removes the oldest entry
        }
        self.entries.push(CacheEntry { prompt, response }); // Adds the new entry to the end
    }
}

#[derive(Serialize)]
struct RequestPayload {
    messages: Vec<Message>,
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
    finish_reason: String,
    index: u32,
}

#[derive(Deserialize, Debug)]
struct ResponsePayload {
    choices: Vec<Choice>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Retrieve the API endpoint and API key from environment variables
    let api_endpoint = env::var("API_ENDPOINT")
        .expect("API_ENDPOINT not set in .env file");
    let api_key = env::var("API_KEY")
        .expect("API_KEY not set in .env file");

    // Ask the user to specify the programming language from a predefined list
    let language = ask_for_language()?;

    // Load the cache from the file
    let mut cache = load_cache("api_cache.json")?;

    loop {
        println!("AI Code Assistant");
        println!("1. Code Completion");
        println!("2. Code Explanation");
        println!("3. Refactoring Suggestions");
        println!("4. Help: How to Use");
        println!("5. Exit");
        print!("Choose an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => code_completion(&api_endpoint, &api_key, &language, &mut cache)?,
            "2" => code_explanation(&api_endpoint, &api_key, &language, &mut cache)?,
            "3" => refactoring_suggestions(&api_endpoint, &api_key, &language, &mut cache)?,
            "4" => help_how_to_use(&api_endpoint, &api_key, &language, &mut cache)?,
            "5" => break,
            _ => println!("Invalid option, please try again."),
        }
    }

    // Save the cache to the file before exiting
    save_cache("api_cache.json", &cache)?;

    Ok(())
}

fn ask_for_language() -> Result<String, Box<dyn std::error::Error>> {
    let valid_languages = vec!["Python", "Rust", "JavaScript", "C++", "Java"];
    loop {
        println!("Please specify the programming language you are using (Python, Rust, JavaScript, C++, Java):");
        print!("Enter your programming language: ");
        io::stdout().flush().unwrap();

        let mut language = String::new();
        io::stdin().read_line(&mut language).unwrap();
        let language = language.trim().to_string();

        if valid_languages.iter().any(|&lang| lang.eq_ignore_ascii_case(&language)) {
            return Ok(language);
        } else {
            println!("Invalid language. Please enter one of the following: Python, Rust, JavaScript, C++, Java.");
        }
    }
}

fn code_completion(api_endpoint: &str, api_key: &str, specified_language: &str, cache: &mut Cache) -> Result<(), Box<dyn std::error::Error>> {
    let code_content = get_code_input()?;
    if !check_language(&code_content, specified_language) {
        println!("The detected language in the code does not match the specified language. Aborting.");
        return Ok(());
    }
    let prompt = format!("You are working with {} code. Your task is to complete the given code:\n\n{}", specified_language, code_content);

    if let Some(entry) = cache.entries.iter().find(|entry| entry.prompt == prompt) {
        println!("Using cached response:\n{}", entry.response);
    } else {
        let request_payload = RequestPayload {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.clone(),
            }],
            temperature: 0.7,
            top_p: 0.95,
            max_tokens: 500, // Increased token limit for code completion
        };

        let response_text = send_api_request(&request_payload, api_endpoint, api_key)?;
        cache.add_entry(prompt, response_text.clone());
        println!("{}", response_text);
    }

    Ok(())
}

fn code_explanation(api_endpoint: &str, api_key: &str, specified_language: &str, cache: &mut Cache) -> Result<(), Box<dyn std::error::Error>> {
    let code_content = get_code_input()?;
    if !check_language(&code_content, specified_language) {
        println!("The detected language in the code does not match the specified language. Aborting.");
        return Ok(());
    }
    let prompt = format!("You are working with {} code. Your task is to explain the following code:\n\n{}", specified_language, code_content);

    if let Some(entry) = cache.entries.iter().find(|entry| entry.prompt == prompt) {
        println!("Using cached response:\n{}", entry.response);
    } else {
        let request_payload = RequestPayload {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.clone(),
            }],
            temperature: 0.7,
            top_p: 0.95,
            max_tokens: 500, // Increased token limit for code explanation
        };

        let response_text = send_api_request(&request_payload, api_endpoint, api_key)?;
        cache.add_entry(prompt, response_text.clone());
        println!("{}", response_text);
    }

    Ok(())
}

fn refactoring_suggestions(api_endpoint: &str, api_key: &str, specified_language: &str, cache: &mut Cache) -> Result<(), Box<dyn std::error::Error>> {
    let code_content = get_code_input()?;
    if !check_language(&code_content, specified_language) {
        println!("The detected language in the code does not match the specified language. Aborting.");
        return Ok(());
    }
    let prompt = format!("You are working with {} code. Your task is to provide refactoring suggestions for the following code:\n\n{}", specified_language, code_content);

    if let Some(entry) = cache.entries.iter().find(|entry| entry.prompt == prompt) {
        println!("Using cached response:\n{}", entry.response);
    } else {
        let request_payload = RequestPayload {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.clone(),
            }],
            temperature: 0.7,
            top_p: 0.95,
            max_tokens: 500, // Increased token limit for refactoring suggestions
        };

        let response_text = send_api_request(&request_payload, api_endpoint, api_key)?;
        cache.add_entry(prompt, response_text.clone());
        println!("{}", response_text);
    }

    Ok(())
}

fn help_how_to_use(api_endpoint: &str, api_key: &str, specified_language: &str, cache: &mut Cache) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = format!("You are working with {} code. Please provide a brief explanation on how to use the features of this AI Code Assistant, including code completion, code explanation, and refactoring suggestions.", specified_language);

    if let Some(entry) = cache.entries.iter().find(|entry| entry.prompt == prompt) {
        println!("Using cached response:\n{}", entry.response);
    } else {
        let request_payload = RequestPayload {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.clone(),
            }],
            temperature: 0.7,
            top_p: 0.95,
            max_tokens: 500, // Increased token limit for help instructions
        };

        let response_text = send_api_request(&request_payload, api_endpoint, api_key)?;
        cache.add_entry(prompt, response_text.clone());
        println!("{}", response_text);
    }

    Ok(())
}

fn get_code_input() -> Result<String, Box<dyn std::error::Error>> {
    println!("Would you like to input the code manually or read it from 'code_input.txt'?");
    println!("1. Manual Input");
    println!("2. Read from 'code_input.txt'");
    print!("Choose an option: ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    match choice.trim() {
        "1" => {
            println!("Enter your code (type 'END' on a new line when finished):");
            let mut code = String::new();
            loop {
                let mut line = String::new();
                io::stdin().read_line(&mut line).unwrap();
                if line.trim() == "END" {
                    break;
                }
                code.push_str(&line);
            }
            Ok(code)
        },
        "2" => {
            let content = fs::read_to_string("code_input.txt")?;
            Ok(content)
        },
        _ => {
            println!("Invalid option, please try again.");
            get_code_input()
        }
    }
}

fn send_api_request(request_payload: &RequestPayload, api_endpoint: &str, api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = ureq::post(api_endpoint)
        .set("Content-Type", "application/json")
        .set("api-key", api_key)
        .send_json(request_payload)?;

    let response_payload: ResponsePayload = response.into_json()?;
    if let Some(choice) = response_payload.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("No response generated.".into())
    }
}

fn check_language(code_content: &str, specified_language: &str) -> bool {
    let detected_language = extract_language_from_code(code_content);
    detected_language.eq_ignore_ascii_case(specified_language)
}

fn extract_language_from_code(code_content: &str) -> String {
    if code_content.contains("#include") {
        "C++".to_string()
    } else if code_content.contains("fn main()") {
        "Rust".to_string()
    } else if code_content.contains("def ") {
        "Python".to_string()
    } else if code_content.contains("function") || code_content.contains("console.log") {
        "JavaScript".to_string()
    } else if code_content.contains("public static void main") {
        "Java".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn load_cache(filename: &str) -> Result<Cache, Box<dyn std::error::Error>> {
    if let Ok(content) = fs::read_to_string(filename) {
        // Try to parse as the new Cache structure
        if let Ok(cache) = serde_json::from_str::<Cache>(&content) {
            Ok(cache)
        } else {
            // If parsing as Cache fails, try to parse as the old HashMap format
            let old_cache: HashMap<String, String> = serde_json::from_str(&content)?;
            let entries = old_cache.into_iter()
                .map(|(prompt, response)| CacheEntry { prompt, response })
                .collect();
            Ok(Cache { entries })
        }
    } else {
        Ok(Cache { entries: Vec::new() }) // If the file doesn't exist, return an empty cache
    }
}

fn save_cache(filename: &str, cache: &Cache) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(filename, content)?;
    Ok(())
}
