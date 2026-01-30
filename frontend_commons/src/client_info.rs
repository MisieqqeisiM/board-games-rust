use backend_commons::client_info::{ClientData, ClientMessage};

pub struct ClientInfo {
    client_info: ts::ClientInfo,
}

impl ClientInfo {
    pub fn new(data: ClientData) -> Self {
        Self {
            client_info: ts::ClientInfo::new(&data.name, data.ping),
        }
    }

    pub fn consume(&mut self, message: ClientMessage) {
        match message {
            ClientMessage::Ping(ping) => self.client_info.setPing(ping),
        }
    }
}

mod ts {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(js_namespace = client_info)]
    unsafe extern "C" {
        pub type ClientInfo;

        #[wasm_bindgen(constructor)]
        pub fn new(name: &str, ping: u32) -> ClientInfo;

        #[wasm_bindgen(method)]
        pub fn setPing(this: &ClientInfo, ping: u32);
    }
}
