pub mod board;

use backend_commons::client_info::ClientListMessage;
use serde::{Deserialize, Serialize};

use crate::board::common::{Board, BoardAction, BoardEvent};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToClient {
    ClientListMessage(ClientListMessage),
    NewBoard(Board<u64>),
    BoardEvent(BoardEvent),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToServer {
    BoardAction(BoardAction),
}
