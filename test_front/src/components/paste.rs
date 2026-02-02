use frontend_commons::command::{Command, CommandConsumer};

#[derive(Debug)]
pub enum PasteCommand {
    File(Vec<u8>),
}

pub struct Paste {
    paste: ts::Paste,
}

impl Paste {
    pub fn new<State, Cmd>(consumer: CommandConsumer<State, PasteCommand, Cmd>) -> Self
    where
        Cmd: Command<State> + 'static,
        State: 'static,
    {
        Self {
            paste: ts::Paste::new(ts::PasteBackend::new(Box::new(consumer))),
        }
    }
}

impl<State, Cmd> ts::PasteObserver for CommandConsumer<State, PasteCommand, Cmd>
where
    Cmd: Command<State>,
{
    fn on_file(&mut self, data: Vec<u8>) {
        self.consume(PasteCommand::File(data));
    }
}

mod ts {
    use wasm_bindgen::prelude::wasm_bindgen;
    use web_sys::js_sys::Uint8Array;

    pub trait PasteObserver {
        fn on_file(&mut self, data: Vec<u8>);
    }

    #[wasm_bindgen]
    pub struct PasteBackend {
        observer: Box<dyn PasteObserver>,
    }

    impl PasteBackend {
        pub fn new(observer: Box<dyn PasteObserver>) -> PasteBackend {
            PasteBackend { observer: observer }
        }
    }

    #[wasm_bindgen]
    impl PasteBackend {
        pub fn on_file(&mut self, data: Uint8Array) {
            self.observer.on_file(data.to_vec());
        }
    }

    #[wasm_bindgen(js_namespace = "paste")]
    unsafe extern "C" {
        pub type Paste;

        #[wasm_bindgen(constructor)]
        pub fn new(backend: PasteBackend) -> Paste;
    }
}
