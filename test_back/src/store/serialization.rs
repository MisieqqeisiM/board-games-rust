use std::io::Result;

use backend_commons::store::{StateBuilder, Store};
use serde::de;

use crate::{
    board,
    store::{
        self,
        v1::{BoardV1, EventV1},
    },
};

// When creating a new version, increment CURRENT_VERSION and add a new variant to the Event and Board enums.

pub const CURRENT_VERSION: u64 = 1;
pub type EventLatest = EventV1;
pub type BoardLatest = BoardV1;

#[derive(Debug)]
enum Event {
    V1(EventV1),
}

#[derive(Debug)]
enum Board {
    V1(BoardV1),
}

pub struct BoardLoader {
    board: BoardV1,
}

pub struct BoardStore<S: Store> {
    pub store: S,
}

impl<S: Store> BoardStore<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn apply_event(&mut self, event: EventLatest) -> Result<()> {
        let data = postcard::to_stdvec(&event).unwrap();
        self.store.apply_event(&data).await
    }

    pub async fn snapshot(&mut self, board: BoardLatest) -> Result<()> {
        let data = postcard::to_stdvec(&board).unwrap();
        self.store.snapshot(&data).await
    }
}

pub fn convert_board(board: crate::board::common::Board<u64>) -> BoardLatest {
    BoardLatest {
        objects: board
            .objects
            .into_iter()
            .map(|(id, obj)| {
                (
                    id,
                    match obj {
                        board::common::BoardObject::Image(image) => store::v1::Object::Image {
                            x: image.x,
                            y: image.y,
                            texture_id: image.texture,
                        },
                        board::common::BoardObject::Line => store::v1::Object::Line,
                    },
                )
            })
            .collect(),
        textures: board
            .textures
            .into_iter()
            .map(|(id, data)| (id, data))
            .collect(),
    }
}

impl BoardLoader {
    pub fn new() -> Self {
        Self {
            board: BoardV1::new(),
        }
    }

    pub fn get_board(self) -> board::common::Board<u64> {
        let objects = self
            .board
            .objects
            .into_iter()
            .map(|(id, obj)| {
                (
                    id,
                    match obj {
                        super::v1::Object::Image { x, y, texture_id } => {
                            board::common::BoardObject::Image(board::common::Image {
                                id,
                                x,
                                y,
                                texture: texture_id,
                            })
                        }
                        super::v1::Object::Line => todo!(),
                    },
                )
            })
            .collect();

        board::common::Board {
            textures: self.board.textures,
            objects,
        }
    }

    fn load_board(&mut self, board: Board) {
        match board {
            Board::V1(b) => self.board = b,
        }
    }

    fn load_board_event(&mut self, event: Event) {
        match event {
            Event::V1(e) => self.board.apply_event(e),
        }
    }
}

impl StateBuilder for BoardLoader {
    fn load_state(&mut self, version: u64, data: Vec<u8>) -> std::io::Result<()> {
        match version {
            1 => {
                let board: BoardV1 = postcard::from_bytes(&data).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to deserialize board: {e}"),
                    )
                })?;
                self.load_board(Board::V1(board));
                Ok(())
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported version: {version}"),
            )),
        }
    }

    fn load_event(&mut self, version: u64, data: Vec<u8>) -> std::io::Result<()> {
        match version {
            1 => {
                let event: EventV1 = postcard::from_bytes(&data).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to deserialize event: {e}"),
                    )
                })?;
                self.load_board_event(Event::V1(event));
                Ok(())
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported version: {version}"),
            )),
        }
    }
}
