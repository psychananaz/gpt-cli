use serde::{Deserialize, Serialize};

mod gpt;
mod ui;

#[derive(Serialize, Deserialize)]
struct Config {
    api_key: String,
    engine: String,
    max_tokens: usize,
    temperature: f32,
    top_p: f32,
}

fn read_config() -> Config {
    if !std::path::Path::new("config.json").exists() {
        let config = Config {
            api_key: "".to_string(),
            engine: "gpt-5.1".to_string(),
            max_tokens: 150,
            temperature: 0.7,
            top_p: 1.0,
        };

        let config_json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write("config.json", config_json).unwrap();
    }

    let config_json = std::fs::read_to_string("config.json").unwrap();
    serde_json::from_str(&config_json).unwrap()
}

fn main() {
    let config = read_config();

    if config.api_key.is_empty() {
        eprintln!("Error: API key is missing. Please add your API key to config.json");
        return;
    }

    let mut chatgpt_client = gpt::GPTClient::new(&config);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut app = ui::App::new();

    if let Err(error) = ui::run(&runtime, &mut chatgpt_client, &mut app) {
        eprintln!("Error running UI: {}", error);
    }
}
