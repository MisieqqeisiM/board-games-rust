use frontend_commons::command::{Command, CommandConsumer};

#[derive(Debug)]
pub enum MouseCommand {
    Move { x: f32, y: f32 },
    Down { button: u8, x: f32, y: f32 },
    Up { button: u8, x: f32, y: f32 },
    Scroll { delta_x: f32, delta_y: f32 },
}

pub struct Mouse {
    mouse: ts::Mouse,
}

impl Mouse {
    pub fn new<State, Cmd>(consumer: CommandConsumer<State, MouseCommand, Cmd>) -> Self
    where
        Cmd: Command<State> + 'static,
        State: 'static,
    {
        Self {
            mouse: ts::Mouse::new(ts::MouseBackend::new(Box::new(consumer))),
        }
    }
}

impl<State, Cmd> ts::MouseObserver for CommandConsumer<State, MouseCommand, Cmd>
where
    Cmd: Command<State>,
{
    fn on_move(&mut self, x: f32, y: f32) {
        self.consume(MouseCommand::Move { x, y });
    }

    fn on_down(&mut self, button: u8, x: f32, y: f32) {
        self.consume(MouseCommand::Down { button, x, y });
    }

    fn on_up(&mut self, button: u8, x: f32, y: f32) {
        self.consume(MouseCommand::Up { button, x, y });
    }

    fn on_scroll(&mut self, delta_x: f32, delta_y: f32) {
        self.consume(MouseCommand::Scroll { delta_x, delta_y });
    }
}

mod ts {
    use wasm_bindgen::prelude::wasm_bindgen;

    pub trait MouseObserver {
        fn on_move(&mut self, x: f32, y: f32);
        fn on_down(&mut self, button: u8, x: f32, y: f32);
        fn on_up(&mut self, button: u8, x: f32, y: f32);
        fn on_scroll(&mut self, delta_x: f32, delta_y: f32);
    }

    #[wasm_bindgen]
    pub struct MouseBackend {
        observer: Box<dyn MouseObserver>,
    }

    impl MouseBackend {
        pub fn new(observer: Box<dyn MouseObserver>) -> MouseBackend {
            MouseBackend { observer: observer }
        }
    }

    #[wasm_bindgen]
    impl MouseBackend {
        pub fn on_move(&mut self, x: f32, y: f32) {
            self.observer.on_move(x, y);
        }

        pub fn on_down(&mut self, button: u8, x: f32, y: f32) {
            self.observer.on_down(button, x, y);
        }

        pub fn on_up(&mut self, button: u8, x: f32, y: f32) {
            self.observer.on_up(button, x, y);
        }

        pub fn on_scroll(&mut self, delta_x: f32, delta_y: f32) {
            self.observer.on_scroll(delta_x, delta_y);
        }
    }

    #[wasm_bindgen(js_namespace = "mouse")]
    unsafe extern "C" {
        pub type Mouse;

        #[wasm_bindgen(constructor)]
        pub fn new(backend: MouseBackend) -> Mouse;
    }
}
