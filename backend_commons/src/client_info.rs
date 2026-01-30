use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientData {
    pub id: u64,
    pub name: String,
    pub ping: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientListMessage {
    Joined(ClientData),
    Quit(u64),
    Update(u64, ClientMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping(u32),
}
