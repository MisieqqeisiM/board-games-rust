use std::collections::HashMap;

use crate::board::common::{
    Board, BoardAction, BoardEvent, BoardObject, Image, ObjectIdentifier, Texture,
};

pub trait BoardObserver {
    fn create_texture(&mut self, data: Vec<u8>) -> Option<u64>;
    fn new_image(&mut self, x: f64, y: f64, texture_id: u64) -> u64;
}

pub struct LocalBoard {
    board: Board<ObjectIdentifier>,
    texture_internal_ids: HashMap<ObjectIdentifier, u64>,
    texture_internal_ids_reverse: HashMap<u64, ObjectIdentifier>,
    image_internal_ids: HashMap<ObjectIdentifier, u64>,
    local_id_counter: u64,
}

impl LocalBoard {
    pub fn new() -> Self {
        Self {
            board: Board {
                objects: HashMap::new(),
                textures: HashMap::new(),
            },
            texture_internal_ids: HashMap::new(),
            image_internal_ids: HashMap::new(),
            local_id_counter: 0,
            texture_internal_ids_reverse: HashMap::new(),
        }
    }

    pub fn load(&mut self, board: Board<u64>, observer: &mut impl BoardObserver) {
        for (texture_global_id, data) in board.textures {
            let internal_id = observer
                .create_texture(data)
                .expect("Texture from the server is always correct");
            let texture_id = ObjectIdentifier::Global(texture_global_id);
            self.init_texture_id(texture_id, internal_id);
        }

        for object in board.objects.into_values() {
            match object {
                BoardObject::Image(Image {
                    id: obj_global_id,
                    x,
                    y,
                    texture: texture_global_id,
                }) => {
                    let texture_id = ObjectIdentifier::Global(texture_global_id);
                    let img_id = ObjectIdentifier::Global(obj_global_id);

                    let texture_internal_id = *self
                        .texture_internal_ids
                        .get(&texture_id)
                        .expect("Texture must exist");

                    let img_internal_id = observer.new_image(x, y, texture_internal_id);
                    self.init_image_id(img_id, img_internal_id);

                    self.board.objects.insert(
                        img_id,
                        BoardObject::Image(Image {
                            id: img_id,
                            x,
                            y,
                            texture: texture_id,
                        }),
                    );
                }
                BoardObject::Line => todo!(),
            }
        }
    }

    fn next_local_id(&mut self) -> u64 {
        self.local_id_counter += 1;
        self.local_id_counter
    }

    fn init_image_id(&mut self, id: ObjectIdentifier, internal_id: u64) {
        self.image_internal_ids.insert(id, internal_id);
    }

    fn update_image_id(&mut self, old_id: ObjectIdentifier, new_id: ObjectIdentifier) {
        let internal_id = self
            .image_internal_ids
            .remove(&old_id)
            .expect("Old image ID must exist");

        self.image_internal_ids.insert(new_id, internal_id);
    }

    fn init_texture_id(&mut self, id: ObjectIdentifier, internal_id: u64) {
        self.texture_internal_ids.insert(id, internal_id);
        self.texture_internal_ids_reverse.insert(internal_id, id);
    }

    fn update_texture_id(&mut self, old_id: ObjectIdentifier, new_id: ObjectIdentifier) {
        let internal_id = self
            .texture_internal_ids
            .remove(&old_id)
            .expect("Old texture ID must exist");

        self.texture_internal_ids.insert(new_id, internal_id);
        self.texture_internal_ids_reverse
            .insert(internal_id, new_id);
    }

    fn create_or_get_texture_id(
        &mut self,
        texture: Texture<u64>,
        observer: &mut impl BoardObserver,
    ) -> u64 {
        match texture {
            Texture::New {
                id: global_id,
                data,
            } => {
                let internal_id = observer
                    .create_texture(data)
                    .expect("Texture from the server is always correct");
                self.init_texture_id(ObjectIdentifier::Global(global_id), internal_id);
                internal_id
            }
            Texture::Existing { id: global_id } => *self
                .texture_internal_ids
                .get(&ObjectIdentifier::Global(global_id))
                .expect("Texture must exist"),
        }
    }

    pub fn apply_event(&mut self, event: BoardEvent, observer: &mut impl BoardObserver) {
        match event {
            BoardEvent::NewImage { id, x, y, texture } => {
                let img_id = ObjectIdentifier::Global(id);
                let texture_id = ObjectIdentifier::Global(texture.get_id());
                let texture_internal_id = self.create_or_get_texture_id(texture, observer);

                let img_internal_id = observer.new_image(x, y, texture_internal_id);
                self.init_image_id(img_id, img_internal_id);

                self.board.objects.insert(
                    img_id,
                    BoardObject::Image(Image {
                        id: img_id,
                        x,
                        y,
                        texture: texture_id,
                    }),
                );
            }
            BoardEvent::ConfirmImage {
                local_id,
                global_id,
                texture_id,
            } => {
                let img_old_id = ObjectIdentifier::Local(local_id);
                let img_new_id = ObjectIdentifier::Global(global_id);
                let texture_new_id = ObjectIdentifier::Global(texture_id);

                let img = self
                    .board
                    .objects
                    .remove(&img_old_id)
                    .expect("Object must exist");

                let BoardObject::Image(img) = img else {
                    panic!("Object must be an image");
                };

                let texture_old_id = img.texture;

                self.update_image_id(img_old_id, img_new_id);
                self.update_texture_id(texture_old_id, texture_new_id);

                self.board.objects.insert(
                    img_new_id,
                    BoardObject::Image(Image {
                        id: img_new_id,
                        texture: texture_new_id,
                        ..img
                    }),
                );
            }
        }
    }

    pub fn new_image(
        &mut self,
        x: f64,
        y: f64,
        data: Vec<u8>,
        observer: &mut impl BoardObserver,
    ) -> Option<BoardAction> {
        let img_local_id = self.next_local_id();
        let img_id = ObjectIdentifier::Local(img_local_id);
        let texture_internal_id = observer.create_texture(data.clone())?;
        let texture_id = self
            .texture_internal_ids_reverse
            .get(&texture_internal_id)
            .cloned();

        let texture = match texture_id {
            Some(texture_id) => Texture::Existing { id: texture_id },
            None => {
                let texture_id = ObjectIdentifier::Local(self.next_local_id());
                self.init_texture_id(texture_id, texture_internal_id);
                Texture::New {
                    id: texture_id,
                    data,
                }
            }
        };

        let img_internal_id = observer.new_image(x, y, texture_internal_id);
        self.init_image_id(img_id, img_internal_id);

        self.board.objects.insert(
            img_id,
            BoardObject::Image(Image {
                id: img_id,
                x,
                y,
                texture: texture.get_id(),
            }),
        );
        Some(BoardAction::NewImage {
            x,
            y,
            local_id: img_local_id,
            texture,
        })
    }
}
