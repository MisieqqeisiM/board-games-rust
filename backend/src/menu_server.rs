use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use menu_back::{
    ToClient, ToServer,
    server_list::{ServerList, ServerListEvent},
};
use tokio::sync::Mutex;
use tracing::info;

use crate::socket_endpoint::{Client, SocketEndpoint, SocketHandler};

pub enum MenuMessage {
    ServerCreated(String),
}

pub struct Menu {
    clients: HashMap<u64, Client<ToClient>>,
    servers: ServerList,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            servers: ServerList::new(),
        }
    }

    async fn broadcast(&mut self, message: ToClient) {
        for client in self.clients.values_mut() {
            client.send(message.clone()).await;
        }
    }
}

impl SocketHandler<ToClient, ToServer, MenuMessage> for Menu {
    async fn on_connect(&mut self, client: Client<ToClient>) {
        let id = client.get_id();
        self.clients.insert(id, client);
        self.broadcast(ToClient::ServerListEvent(ServerListEvent::Set(
            self.servers.clone(),
        )))
        .await;
    }

    async fn on_message(&mut self, client_id: u64, message: ToServer) {
        match message {
            ToServer::Pong => {
                info!("Pong from {}", client_id);
            }
        };
    }

    async fn on_disconnect(&mut self, client_id: u64) {
        self.clients.remove(&client_id);
    }

    async fn on_internal_message(&mut self, internal_message: MenuMessage) {
        match internal_message {
            MenuMessage::ServerCreated(path) => {
                let event = ServerListEvent::Add(path);
                self.servers.apply(event.clone());
                self.broadcast(ToClient::ServerListEvent(event)).await
            }
        }
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
        let Some(client) = self.clients.get(&client_id) else {
            return;
        };

        info!(
            "Client {} ping: {}ms",
            client.get_user_data().username,
            now.checked_sub(timestamp).unwrap_or(0)
        );
    }
}
