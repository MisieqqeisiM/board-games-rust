use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::response::sse::Event;
use backend_commons::client_info::{ClientData, ClientListMessage, ClientMessage};
use test_back::{
    ToClient, ToServer,
    board::{BoardEvent, EventSender, GlobalBoard},
};
use tracing::info;

use crate::socket_endpoint::{Client, SocketHandler};

pub struct Test {
    clients: Clients,
    board: GlobalBoard,
}

struct Clients {
    clients: HashMap<u64, Client<ToClient>>,
}

impl EventSender for Clients {
    async fn send_event(&mut self, client_id: u64, event: BoardEvent) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            let message = ToClient::BoardEvent(event);
            client.send(message).await;
        }
    }
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u64, client: Client<ToClient>) {
        self.clients.insert(id, client);
    }

    pub fn remove(&mut self, id: &u64) {
        self.clients.remove(id);
    }

    pub fn get_mut(&mut self, id: &u64) -> Option<&mut Client<ToClient>> {
        self.clients.get_mut(id)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Client<ToClient>> {
        self.clients.values_mut()
    }
}

impl Test {
    pub fn new() -> Self {
        Self {
            clients: Clients::new(),
            board: GlobalBoard::new(),
        }
    }

    async fn broadcast(&mut self, message: ToClient) {
        for client in self.clients.values_mut() {
            client.send(message.clone()).await;
        }
    }
}

impl SocketHandler<ToClient, ToServer, ()> for Test {
    async fn on_connect(&mut self, mut client: Client<ToClient>) {
        let id = client.get_id();
        let user_data = client.get_user_data();
        self.board.new_client(id);
        client
            .send(ToClient::NewBoard(self.board.get_state()))
            .await;
        self.clients.insert(id, client);
        self.broadcast(ToClient::ClientListMessage(ClientListMessage::Joined(
            ClientData {
                id,
                name: user_data.username,
                ping: 0,
            },
        )))
        .await;
    }

    async fn on_message(&mut self, client_id: u64, message: ToServer) {
        match message {
            ToServer::BoardAction(action) => {
                self.board.apply(client_id, action, &mut self.clients).await;
            }
        }
    }

    async fn on_disconnect(&mut self, client_id: u64) {
        self.clients.remove(&client_id);
        self.broadcast(ToClient::ClientListMessage(ClientListMessage::Quit(
            client_id,
        )))
        .await;
    }

    async fn tick(&mut self) {
        info!("tick!");
        for client in self.clients.values_mut() {
            client.ping().await;
        }
    }

    async fn on_ping(&mut self, client_id: u64, data: axum::body::Bytes) {
        let Some(client) = self.clients.get_mut(&client_id) else {
            return;
        };
        client.pong(data).await
    }

    async fn on_pong(&mut self, client_id: u64, timestamp: u128) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let ping = now.checked_sub(timestamp).unwrap_or(0) as u32;
        self.broadcast(ToClient::ClientListMessage(ClientListMessage::Update(
            client_id,
            ClientMessage::Ping(ping),
        )))
        .await;
    }

    async fn on_internal_message(&mut self, _: ()) {}
}
