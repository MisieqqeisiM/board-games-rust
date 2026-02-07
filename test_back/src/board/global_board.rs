use std::collections::HashMap;

use crate::board::common::{
    Board, BoardAction, BoardEvent, BoardObject, Image, ObjectIdentifier, Texture,
};

pub struct GlobalBoard {
    board: Board<u64>,
    rev_textures: HashMap<Vec<u8>, u64>,
    global_id_counter: u64,
    clients: HashMap<u64, Client>,
}

pub trait EventSender {
    fn send_event(&mut self, client_id: u64, event: BoardEvent) -> impl Future<Output = ()>;
}

pub trait BoardObserver {
    fn new_image(
        &mut self,
        id: u64,
        x: f64,
        y: f64,
        texture: Texture<u64>,
    ) -> impl Future<Output = ()>;
}

impl GlobalBoard {
    pub fn from_board(board: Board<u64>) -> Self {
        let rev_textures = board
            .textures
            .iter()
            .map(|(id, data)| (data.clone(), *id))
            .collect();
        let global_id_counter = board
            .objects
            .keys()
            .chain(board.textures.keys())
            .max()
            .cloned()
            .unwrap_or(0);
        Self {
            board,
            rev_textures,
            global_id_counter,
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
        observer: &mut impl BoardObserver,
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
                    BoardObject::Image(Image {
                        id: global_id,
                        x,
                        y,
                        texture: texture_global.get_id(),
                    }),
                );

                observer
                    .new_image(global_id, x, y, texture_global.clone())
                    .await;

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
