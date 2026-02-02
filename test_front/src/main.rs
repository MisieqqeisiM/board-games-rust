mod camera;
mod components;
mod image_atlas;
mod textures;

use std::{collections::HashMap, vec};

use frontend_commons::{
    client_info::ClientInfo,
    command::{Command, CommandConsumerGenerator},
    socket::{Socket, SocketCommand},
};
use log::{Level, debug, info};
use test_back::{
    ToClient, ToServer,
    board::local_board::{BoardObserver, LocalBoard},
};

use crate::{
    camera::Camera,
    components::{
        canvas::Canvas,
        mouse::{Mouse, MouseCommand},
        paste::{Paste, PasteCommand},
    },
    image_atlas::BoundingBox,
    textures::Textures,
};

struct Graphics {
    canvas: Canvas,
    textures: Textures,
    local_id_counter: u64,
}

impl Graphics {
    pub fn next_local_id(&mut self) -> u64 {
        let id = self.local_id_counter;
        self.local_id_counter += 1;
        id
    }
}

impl BoardObserver for Graphics {
    fn create_texture(&mut self, data: Vec<u8>) -> Option<u64> {
        self.textures.insert_texture(data, &mut self.canvas)
    }

    fn new_image(&mut self, x: f64, y: f64, texture_id: u64) -> u64 {
        let bounding_box = self.textures.get_bounds(texture_id);
        let (width, height) = if bounding_box.rotated {
            (bounding_box.height, bounding_box.width)
        } else {
            (bounding_box.width, bounding_box.height)
        };
        self.canvas.push(
            bounding_box.atlas_id / 8,
            get_vertices(&bounding_box, x as f32, y as f32, width, height),
        );
        self.next_local_id()
    }
}

struct TestState {
    socket: Socket<ToClient, ToServer>,
    clients: HashMap<u64, ClientInfo>,
    paste: Paste,
    mouse: Mouse,
    camera: Camera,
    graphics: Graphics,
    board: LocalBoard,
}

#[derive(Debug)]
enum TestCommand {
    Socket(SocketCommand<ToClient>),
    Paste(PasteCommand),
    Mouse(MouseCommand),
}

fn get_vertices(
    bounding_box: &BoundingBox,
    x: f32,
    y: f32,
    img_width: u32,
    img_height: u32,
) -> Vec<f32> {
    let atlas_id = (bounding_box.atlas_id % 8) as f32;
    let (width, height, bb_x, bb_y, bb_w, bb_h) = (
        img_width as f32,
        img_height as f32,
        bounding_box.x as f32 / 2048.0,
        bounding_box.y as f32 / 2048.0,
        bounding_box.width as f32 / 2048.0,
        bounding_box.height as f32 / 2048.0,
    );
    let (v1, v2, v3, v4) = if bounding_box.rotated {
        (
            vec![x, y, bb_x + bb_w, bb_y, atlas_id],
            vec![x, y + height, bb_x, bb_y, atlas_id],
            vec![x + width, y + height, bb_x, bb_y + bb_h, atlas_id],
            vec![x + width, y, bb_x + bb_w, bb_y + bb_h, atlas_id],
        )
    } else {
        (
            vec![x, y, bb_x, bb_y, atlas_id],
            vec![x, y + height, bb_x, bb_y + bb_h, atlas_id],
            vec![x + width, y + height, bb_x + bb_w, bb_y + bb_h, atlas_id],
            vec![x + width, y, bb_x + bb_w, bb_y, atlas_id],
        )
    };
    [v1.clone(), v2, v3.clone(), v1, v3, v4].concat()
}

impl Command<TestState> for TestCommand {
    fn apply(self, state: &mut TestState) {
        match self {
            TestCommand::Paste(PasteCommand::File(data)) => {
                let (x, y) = state.camera.get_mouse_position();
                let action = state.board.new_image(x, y, data, &mut state.graphics);
                if let Some(action) = action {
                    state.socket.send(ToServer::BoardAction(action));
                };
                state.graphics.canvas.draw();
            }
            TestCommand::Mouse(mouse_command) => {
                if state.camera.update(mouse_command) {
                    state.graphics.canvas.set_transform(
                        state.camera.get_x() as f32,
                        state.camera.get_y() as f32,
                        state.camera.get_scale() as f32,
                    );
                    state.graphics.canvas.draw();
                }
            }
            TestCommand::Socket(SocketCommand::Data(event)) => {
                debug!("{:?}", event);
                match event {
                    ToClient::BoardEvent(board_event) => {
                        state.board.apply_event(board_event, &mut state.graphics);
                        state.graphics.canvas.draw();
                    }
                    ToClient::NewBoard(board) => {
                        state.board.load(board, &mut state.graphics);
                        state.graphics.canvas.draw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl From<SocketCommand<ToClient>> for TestCommand {
    fn from(value: SocketCommand<ToClient>) -> Self {
        TestCommand::Socket(value)
    }
}

impl From<PasteCommand> for TestCommand {
    fn from(value: PasteCommand) -> Self {
        TestCommand::Paste(value)
    }
}

impl From<MouseCommand> for TestCommand {
    fn from(value: MouseCommand) -> Self {
        TestCommand::Mouse(value)
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
    info!("Hello, world!");
    let mut consumer_generator: CommandConsumerGenerator<_, TestCommand> =
        CommandConsumerGenerator::new();
    let state = TestState {
        socket: Socket::new(consumer_generator.make_consumer(), "socket"),
        clients: HashMap::new(),
        graphics: Graphics {
            canvas: Canvas::new(),
            textures: Textures::new(),
            local_id_counter: 0,
        },
        paste: Paste::new(consumer_generator.make_consumer()),
        mouse: Mouse::new(consumer_generator.make_consumer()),
        camera: Camera::new(),
        board: LocalBoard::new(),
    };
    consumer_generator.activate(state);
}
