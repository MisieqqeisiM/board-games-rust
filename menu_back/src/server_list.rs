use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerList {
    pub servers: HashSet<String>,
}

impl ServerList {
    pub fn new() -> ServerList {
        ServerList {
            servers: HashSet::new(),
        }
    }

    pub fn apply(&mut self, event: ServerListEvent) {
        match event {
            ServerListEvent::Set(list) => self.servers = list.servers,
            ServerListEvent::Add(server) => {
                self.servers.insert(server);
            }
            ServerListEvent::Remove(server) => {
                self.servers.remove(&server);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerListEvent {
    Set(ServerList),
    Add(String),
    Remove(String),
}
