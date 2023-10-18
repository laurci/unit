use serde::{de::DeserializeOwned, Deserialize, Serialize};
use unit_utils::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
}

impl WsMessage {
    pub fn to_string(self) -> String {
        match self {
            WsMessage::Text(text) => text,
            WsMessage::Binary(data) => String::from_utf8(data).unwrap(),
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            WsMessage::Text(text) => text.into_bytes(),
            WsMessage::Binary(data) => data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CrossbarContent {
    Text(String),
    Binary(Vec<u8>),
}

impl CrossbarContent {
    pub fn to_string(self) -> String {
        match self {
            CrossbarContent::Text(text) => text,
            CrossbarContent::Binary(data) => String::from_utf8(data).unwrap(),
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            CrossbarContent::Text(text) => text.into_bytes(),
            CrossbarContent::Binary(data) => data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossbarMessage {
    pub topic: String,
    pub content: CrossbarContent,
}

pub fn encode_runtime_proto_message<T>(message: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let data = bincode::serialize(&message)?;
    Ok(data)
}

pub fn decode_runtime_proto_message<T>(data: Vec<u8>) -> Result<T>
where
    T: DeserializeOwned,
{
    let message = bincode::deserialize(&data)?;
    Ok(message)
}
