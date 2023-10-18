use serde::{Deserialize, Serialize};
use unit_utils::Result;

pub static CROSSBAR_TOPIC: &str = "crossbar";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CrossbarContent {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossbarMessage {
    pub topic: String,
    pub content: CrossbarContent,
}

impl CrossbarMessage {
    pub fn text(topic: String, message: String) -> Self {
        Self {
            topic,
            content: CrossbarContent::Text(message),
        }
    }

    pub fn binary(topic: String, message: Vec<u8>) -> Self {
        Self {
            topic,
            content: CrossbarContent::Binary(message),
        }
    }
}

pub fn encode_crossbar_message(message: CrossbarMessage) -> Result<Vec<u8>> {
    let data = bincode::serialize(&message)?;
    Ok(data)
}

pub fn decode_crossbar_message(data: Vec<u8>) -> Result<CrossbarMessage> {
    let message = bincode::deserialize(&data)?;
    Ok(message)
}
