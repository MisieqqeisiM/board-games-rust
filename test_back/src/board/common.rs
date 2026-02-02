use serde::{Deserialize, Serialize, de::DeserializeOwned};

use std::{collections::HashMap, hash::Hash};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BoardAction {
    NewImage {
        x: f64,
        y: f64,
        local_id: u64,
        texture: Texture<ObjectIdentifier>,
    },
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BoardObject<Id> {
    Image(Image<Id>),
    // Todo: implement lines
    Line,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Image<Id> {
    pub id: Id,
    pub x: f64,
    pub y: f64,
    pub texture: Id,
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
