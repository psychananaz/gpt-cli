use reqwest::{Client, Error};
use serde_json::{json, Value};

pub struct GPTClient {
    client: Client,
    engine: String,
    api_key: String,
    max_tokens: usize,
    temperature: f32,
    top_p: f32,

    message_history: Vec<Value>,
}

impl GPTClient {
    pub fn new(config: &super::Config) -> GPTClient {
        GPTClient {
            client: Client::new(),
            api_key: config.api_key.to_string(),
            message_history: vec![json!({
                "role": "system",
                "content": [{"type": "input_text", "text": "You are an assistant integrated into a terminal UI. Keep responses concise and in plain text."}]
            })],

            engine: config.engine.to_string(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            top_p: config.top_p,
        }
    }

    pub async fn get_response(&mut self, user_input: &str) -> Result<String, Error> {
        self.message_history.push(json!({
            "role": "user",
            "content": [
                {"type": "input_text", "text": user_input}
            ]
        }));

        let body = json!({
            "model": self.engine,
            "input": self.message_history,
            "max_output_tokens": self.max_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
            "response_format": {"type": "text"},
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let ai_response = response["output"][0]["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if ai_response.is_empty() {
            eprintln!("Error: Invalid model name in config.json or empty response received");
        }

        self.message_history.push(json!({
            "role": "assistant",
            "content": [
                {"type": "output_text", "text": &ai_response}
            ]
        }));

        if self.message_history.len() > 11 {
            self.message_history.remove(1);
        }

        Ok(ai_response)
    }
}
