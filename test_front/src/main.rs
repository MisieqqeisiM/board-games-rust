mod camera;
mod canvas;
mod image_atlas;
mod mouse;
mod paste;
mod textures;

use std::{
    collections::HashMap,
    io::{BufReader, Cursor},
    vec,
};

use frontend_commons::{
    client_info::ClientInfo,
    command::{Command, CommandConsumerGenerator},
    list::{List, ListCommand},
    socket::{Socket, SocketCommand},
};
use image::{DynamicImage, ImageReader, imageops::FilterType::Nearest};
use log::{Level, debug, error, info};
use test_back::{
    ToClient, ToServer,
    board::{self, BoardAction, BoardEvent, ObjectIdentifier, Texture},
};

use crate::{
    camera::Camera,
    canvas::Canvas,
    image_atlas::{BoundingBox, ImageAtlas},
    mouse::{Mouse, MouseCommand},
    paste::{Paste, PasteCommand},
    textures::Textures,
};

struct TestState {
    socket: Socket<ToClient, ToServer>,
    clients: HashMap<u64, ClientInfo>,
    canvas: Canvas,
    paste: Paste,
    mouse: Mouse,
    textures: Textures,
    camera: Camera,
    local_id_counter: u64,
    rect_to_texture: HashMap<ObjectIdentifier, ObjectIdentifier>,
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
    let v1 = vec![
        x,
        y,
        bounding_box.x as f32 / 2048.0,
        bounding_box.y as f32 / 2048.0,
        bounding_box.atlas_id as f32,
    ];
    let v2 = vec![
        x,
        y + img_height as f32,
        bounding_box.x as f32 / 2048.0,
        bounding_box.y as f32 / 2048.0 + img_height as f32 / 2048.0,
        bounding_box.atlas_id as f32,
    ];
    let v3 = vec![
        x + img_width as f32,
        y + img_height as f32,
        bounding_box.x as f32 / 2048.0 + img_width as f32 / 2048.0,
        bounding_box.y as f32 / 2048.0 + img_height as f32 / 2048.0,
        bounding_box.atlas_id as f32,
    ];
    let v4 = vec![
        x + img_width as f32,
        y,
        bounding_box.x as f32 / 2048.0 + img_width as f32 / 2048.0,
        bounding_box.y as f32 / 2048.0,
        bounding_box.atlas_id as f32,
    ];

    [v1.clone(), v2, v3.clone(), v1, v3, v4].concat()
}

impl TestState {
    pub fn next_local_id(&mut self) -> u64 {
        let id = self.local_id_counter;
        self.local_id_counter += 1;
        id
    }
}

impl Command<TestState> for TestCommand {
    fn apply(self, state: &mut TestState) {
        match self {
            TestCommand::Paste(PasteCommand::File(data)) => {
                let tex = match state.textures.get_texture_id(&data) {
                    Some(obj_id) => Texture::Existing { id: obj_id },
                    None => {
                        let obj_id = ObjectIdentifier::Local(state.next_local_id());
                        state
                            .textures
                            .insert_texture(obj_id, data.clone(), &mut state.canvas);
                        Texture::New { id: obj_id, data }
                    }
                };
                let bounding_box = state.textures.get_bounds(tex.get_id());
                let (x, y) = state.camera.get_mouse_position();
                let rect_id = state.next_local_id();
                state.canvas.push(get_vertices(
                    &bounding_box,
                    x as f32,
                    y as f32,
                    bounding_box.width,
                    bounding_box.height,
                ));
                state
                    .rect_to_texture
                    .insert(ObjectIdentifier::Local(rect_id), tex.get_id());
                state
                    .socket
                    .send(ToServer::BoardAction(BoardAction::NewImage {
                        x,
                        y,
                        local_id: rect_id,
                        texture: tex,
                    }));
                state.canvas.draw();
            }
            TestCommand::Mouse(mouse_command) => {
                state.camera.update(mouse_command);
                state.canvas.set_transform(
                    state.camera.get_x() as f32,
                    state.camera.get_y() as f32,
                    state.camera.get_scale() as f32,
                );
                state.canvas.draw();
            }
            TestCommand::Socket(SocketCommand::Data(event)) => {
                debug!("{:?}", event);
                match event {
                    ToClient::BoardEvent(board_event) => match board_event {
                        BoardEvent::NewImage { id, x, y, texture } => {
                            let obj_id = ObjectIdentifier::Global(texture.get_id());
                            match texture {
                                Texture::New { id: _, data } => {
                                    if !state.textures.texture_exists(&data) {
                                        state.textures.insert_texture(
                                            obj_id,
                                            data.clone(),
                                            &mut state.canvas,
                                        );
                                    }
                                }
                                Texture::Existing { id: _ } => {}
                            }
                            let bounding_box = state.textures.get_bounds(obj_id);
                            state.canvas.push(get_vertices(
                                &bounding_box,
                                x as f32,
                                y as f32,
                                bounding_box.width,
                                bounding_box.height,
                            ));
                            state
                                .rect_to_texture
                                .insert(ObjectIdentifier::Global(id), obj_id);
                            state.canvas.draw();
                        }
                        BoardEvent::ConfirmImage {
                            local_id,
                            global_id,
                            texture_id,
                        } => {
                            let old_tex_id = state
                                .rect_to_texture
                                .remove(&ObjectIdentifier::Local(local_id))
                                .unwrap();
                            state.rect_to_texture.insert(
                                ObjectIdentifier::Global(global_id),
                                ObjectIdentifier::Global(texture_id),
                            );
                            state
                                .textures
                                .update_id(old_tex_id, ObjectIdentifier::Global(texture_id));
                        }
                    },
                    ToClient::NewBoard(board) => {
                        for (id, tex) in board.textures.iter() {
                            let obj_id = ObjectIdentifier::Global(*id);
                            state
                                .textures
                                .insert_texture(obj_id, tex.clone(), &mut state.canvas);
                        }
                        for (id, object) in board.objects.iter() {
                            match object {
                                test_back::board::BoardObject::Image { x, y, texture, .. } => {
                                    let obj_id = ObjectIdentifier::Global(*id);
                                    let tex_id = ObjectIdentifier::Global(*texture);
                                    state.rect_to_texture.insert(obj_id, tex_id);
                                    let bounding_box = state.textures.get_bounds(tex_id);
                                    state.canvas.push(get_vertices(
                                        &bounding_box,
                                        *x as f32,
                                        *y as f32,
                                        bounding_box.width,
                                        bounding_box.height,
                                    ));
                                }
                            }
                        }
                        state.canvas.draw();
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
        canvas: Canvas::new(),
        paste: Paste::new(consumer_generator.make_consumer()),
        mouse: Mouse::new(consumer_generator.make_consumer()),
        textures: Textures::new(),
        camera: Camera::new(),
        local_id_counter: 0,
        rect_to_texture: HashMap::new(),
    };
    consumer_generator.activate(state);
}
