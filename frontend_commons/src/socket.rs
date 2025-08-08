use std::marker::PhantomData;

use serde::{Serialize, de::DeserializeOwned};

use crate::command::{Command, CommandConsumer};

#[derive(Debug)]
pub enum SocketCommand<ToClient> {
    Data(ToClient),
}

pub struct Socket<ToClient, ToServer> {
    socket: ts::Socket,
    to_client: PhantomData<ToClient>,
    to_server: PhantomData<ToServer>,
}

impl<ToClient, ToServer> Socket<ToClient, ToServer>
where
    ToServer: Serialize,
    ToClient: DeserializeOwned + 'static,
{
    pub fn new<State, Cmd>(
        consumer: CommandConsumer<State, SocketCommand<ToClient>, Cmd>,
        path: &str,
    ) -> Self
    where
        Cmd: Command<State> + 'static,
        State: 'static,
    {
        Self {
            socket: ts::Socket::new(ts::SocketBackend::new(Box::new(consumer)), path),
            to_client: PhantomData,
            to_server: PhantomData,
        }
    }

    pub fn send(&mut self, message: ToServer) {
        self.socket.send(serde_cbor::to_vec(&message).unwrap());
    }
}

impl<State, Cmd, ToClient> ts::SocketObserver
    for CommandConsumer<State, SocketCommand<ToClient>, Cmd>
where
    Cmd: Command<State>,
    ToClient: DeserializeOwned,
{
    fn on_data(&mut self, data: Vec<u8>) {
        self.consume(SocketCommand::Data(serde_cbor::from_slice(&data).unwrap()));
    }
}

mod ts {
    use wasm_bindgen::prelude::wasm_bindgen;
    use web_sys::js_sys::Uint8Array;

    pub trait SocketObserver {
        fn on_data(&mut self, data: Vec<u8>);
    }

    #[wasm_bindgen]
    pub struct SocketBackend {
        observer: Box<dyn SocketObserver>,
    }

    impl SocketBackend {
        pub fn new(observer: Box<dyn SocketObserver>) -> SocketBackend {
            SocketBackend { observer: observer }
        }
    }

    #[wasm_bindgen]
    impl SocketBackend {
        pub fn on_data(&mut self, data: Uint8Array) {
            self.observer.on_data(data.to_vec());
        }
    }

    #[wasm_bindgen(js_namespace = socket)]
    unsafe extern "C" {
        pub type Socket;

        #[wasm_bindgen(constructor)]
        pub fn new(backend: SocketBackend, path: &str) -> Socket;

        #[wasm_bindgen(method)]
        pub fn send(this: &Socket, data: Vec<u8>);

    }
}
