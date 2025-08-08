use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToClient {
    Ping,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToServer {
    Pong,
}
