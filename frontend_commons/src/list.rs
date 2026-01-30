use crate::command::{Command, CommandConsumer};

#[derive(Debug)]
pub enum ListCommand {
    Click(u32),
}

pub struct List {
    list: ts::List,
}

impl List {
    pub fn new<State, Cmd>(consumer: CommandConsumer<State, ListCommand, Cmd>) -> Self
    where
        Cmd: Command<State> + 'static,
        State: 'static,
    {
        Self {
            list: ts::List::new(ts::ListBackend::new(Box::new(consumer))),
        }
    }

    pub fn add_element(&mut self, content: &str) -> u32 {
        self.list.add_element(content)
    }

    pub fn remove_element(&mut self, elem: u32) {
        self.list.remove_element(elem)
    }
}

impl<State, Cmd> ts::ListObserver for CommandConsumer<State, ListCommand, Cmd>
where
    Cmd: Command<State>,
{
    fn on_click(&mut self, elem: u32) {
        self.consume(ListCommand::Click(elem));
    }
}

mod ts {
    use wasm_bindgen::prelude::wasm_bindgen;

    pub trait ListObserver {
        fn on_click(&mut self, elem: u32);
    }

    #[wasm_bindgen]
    pub struct ListBackend {
        observer: Box<dyn ListObserver>,
    }

    impl ListBackend {
        pub fn new(observer: Box<dyn ListObserver>) -> ListBackend {
            ListBackend { observer }
        }
    }

    #[wasm_bindgen]
    impl ListBackend {
        pub fn on_click(&mut self, elem: u32) {
            self.observer.on_click(elem);
        }
    }

    #[wasm_bindgen(js_namespace = list)]
    unsafe extern "C" {
        pub type List;

        #[wasm_bindgen(constructor)]
        pub fn new(backend: ListBackend) -> List;

        #[wasm_bindgen(method)]
        pub fn add_element(this: &List, content: &str) -> u32;

        #[wasm_bindgen(method)]
        pub fn remove_element(this: &List, elem: u32);
    }
}
