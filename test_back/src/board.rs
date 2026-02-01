use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BoardObject<Id> {
    Image { id: Id, x: f64, y: f64, texture: Id },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum ObjectIdentifier {
    Local(u64),
    Global(u64),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "Id: Serialize + DeserializeOwned + Eq + Hash")]
pub struct Board<Id> {
    pub objects: HashMap<Id, BoardObject<Id>>,
    pub textures: HashMap<Id, Vec<u8>>,
}

struct Client {
    id: u64,
    ids_map: HashMap<u64, u64>,
}

impl Client {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ids_map: HashMap::new(),
        }
    }

    pub fn set_global_id(&mut self, local_id: u64, global_id: u64) {
        self.ids_map.insert(local_id, global_id);
    }

    pub fn get_global_id(&self, id: ObjectIdentifier) -> Option<u64> {
        match id {
            ObjectIdentifier::Local(local_id) => self.ids_map.get(&local_id).cloned(),
            ObjectIdentifier::Global(global_id) => Some(global_id),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BoardAction {
    NewImage {
        x: f64,
        y: f64,
        local_id: u64,
        texture: Texture<ObjectIdentifier>,
    },
}

pub struct GlobalBoard {
    board: Board<u64>,
    rev_textures: HashMap<Vec<u8>, u64>,
    global_id_counter: u64,
    clients: HashMap<u64, Client>,
}

pub trait EventSender {
    fn send_event(&mut self, client_id: u64, event: BoardEvent) -> impl Future<Output = ()>;
}

impl GlobalBoard {
    pub fn new() -> Self {
        Self {
            board: Board {
                objects: HashMap::new(),
                textures: HashMap::new(),
            },
            rev_textures: HashMap::new(),
            global_id_counter: 0,
            clients: HashMap::new(),
        }
    }

    pub fn get_state(&self) -> Board<u64> {
        self.board.clone()
    }

    pub fn new_client(&mut self, client_id: u64) {
        self.clients.insert(client_id, Client::new(client_id));
    }

    fn get_global_texture(
        &mut self,
        client_id: u64,
        data: &Texture<ObjectIdentifier>,
    ) -> Option<Texture<u64>> {
        match data {
            Texture::New { id: _, data } => {
                if let Some(&existing_id) = self.rev_textures.get(data) {
                    return Some(Texture::Existing { id: existing_id });
                }
                let global_id = self.next_global_id();
                self.board.textures.insert(global_id, data.clone());
                self.rev_textures.insert(data.clone(), global_id);
                Some(Texture::New {
                    id: global_id,
                    data: data.to_owned(),
                })
            }
            Texture::Existing { id } => self.clients.get(&client_id).and_then(|client| {
                client
                    .get_global_id(*id)
                    .map(|global_id| Texture::Existing { id: global_id })
            }),
        }
    }

    fn next_global_id(&mut self) -> u64 {
        self.global_id_counter += 1;
        self.global_id_counter
    }

    pub async fn apply(
        &mut self,
        client_id: u64,
        board_action: BoardAction,
        event_sender: &mut impl EventSender,
    ) {
        match board_action {
            BoardAction::NewImage {
                x,
                y,
                local_id,
                texture,
            } => {
                let global_id = self.next_global_id();
                let Some(texture_global) = self.get_global_texture(client_id, &texture) else {
                    return;
                };
                self.board.objects.insert(
                    global_id,
                    BoardObject::Image {
                        id: global_id,
                        x,
                        y,
                        texture: texture_global.get_id(),
                    },
                );
                for client in self.clients.values_mut() {
                    if client.id == client_id {
                        client.set_global_id(local_id, global_id);
                        if let ObjectIdentifier::Local(local_texture_id) = texture.get_id() {
                            client.set_global_id(local_texture_id, texture_global.get_id());
                        }
                        event_sender
                            .send_event(
                                client_id,
                                BoardEvent::ConfirmImage {
                                    local_id,
                                    global_id,
                                    texture_id: texture_global.get_id(),
                                },
                            )
                            .await;
                    } else {
                        event_sender
                            .send_event(
                                client.id,
                                BoardEvent::NewImage {
                                    id: global_id,
                                    x,
                                    y,
                                    texture: texture_global.clone(),
                                },
                            )
                            .await;
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Texture<Id> {
    New { id: Id, data: Vec<u8> },
    Existing { id: Id },
}

impl<Id: Clone + Copy> Texture<Id> {
    pub fn get_id(&self) -> Id {
        match self {
            Texture::New { id, data: _ } => *id,
            Texture::Existing { id } => *id,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BoardEvent {
    NewImage {
        id: u64,
        x: f64,
        y: f64,
        texture: Texture<u64>,
    },
    ConfirmImage {
        local_id: u64,
        global_id: u64,
        texture_id: u64,
    },
}
