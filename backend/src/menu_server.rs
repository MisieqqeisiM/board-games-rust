use std::collections::HashMap;

use menu_back::{ToClient, ToServer};
use tracing::info;

use crate::socket_endpoint::{Client, SocketHandler};

pub struct Menu {
    clients: HashMap<u64, Client<ToClient>>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    async fn broadcast(&mut self, message: ToClient) {
        for client in self.clients.values_mut() {
            client.send(message.clone()).await;
        }
    }
}

impl SocketHandler<ToClient, ToServer> for Menu {
    async fn on_connect(&mut self, client: Client<ToClient>) {
        let id = client.get_id();
        self.clients.insert(id, client);
        self.broadcast(ToClient::Ping).await;
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

    async fn tick(&mut self) {
        info!("tick!");
    }
}
