use backend_commons::client_info::ClientListMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToClient {
    ClientListMessage(ClientListMessage),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToServer {}
