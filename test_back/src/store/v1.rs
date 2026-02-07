use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// DO NOT CHANGE AFTER RELEASE

#[derive(Serialize, Deserialize, Debug)]
pub enum Object {
    Image { x: f64, y: f64, texture_id: u64 },
    Line,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardV1 {
    pub textures: HashMap<u64, Vec<u8>>,
    pub objects: HashMap<u64, Object>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Texture {
    New { id: u64, data: Vec<u8> },
    Existing { id: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventV1 {
    NewImage {
        id: u64,
        x: f64,
        y: f64,
        texture: Texture,
    },
}

impl BoardV1 {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    pub fn apply_event(&mut self, event: EventV1) {
        match event {
            EventV1::NewImage { id, x, y, texture } => {
                let texture_id = match texture {
                    Texture::New { id: tex_id, data } => {
                        self.textures.insert(tex_id, data);
                        tex_id
                    }
                    Texture::Existing { id: tex_id } => tex_id,
                };
                self.objects.insert(id, Object::Image { x, y, texture_id });
            }
        }
    }
}
