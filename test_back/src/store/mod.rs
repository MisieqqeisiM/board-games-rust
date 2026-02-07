use backend_commons::store::Store;

use crate::{
    board::global_board::BoardObserver,
    store::{
        serialization::{BoardStore, convert_board},
        v1::EventV1,
    },
};

pub mod serialization;
pub mod v1;

pub struct StoringObserver<S: Store> {
    store: BoardStore<S>,
}

impl<S: Store> StoringObserver<S> {
    pub fn new(store: S) -> Self {
        Self {
            store: BoardStore::new(store),
        }
    }

    pub fn get_store_mut(&mut self) -> &mut S {
        &mut self.store.store
    }

    pub async fn snapshot(&mut self, board: crate::board::common::Board<u64>) {
        let board = convert_board(board);
        self.store.snapshot(board).await.unwrap();
    }
}

impl<S: Store> BoardObserver for StoringObserver<S> {
    async fn new_image(
        &mut self,
        id: u64,
        x: f64,
        y: f64,
        texture: crate::board::common::Texture<u64>,
    ) {
        let texture = match texture {
            crate::board::common::Texture::New { id, data } => v1::Texture::New { id, data },
            crate::board::common::Texture::Existing { id: tex_id } => {
                v1::Texture::Existing { id: tex_id }
            }
        };
        let event = EventV1::NewImage { id, x, y, texture };
        self.store.apply_event(event).await.unwrap();
    }
}
