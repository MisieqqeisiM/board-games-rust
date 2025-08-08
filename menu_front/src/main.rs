use std::marker::PhantomData;

use frontend_commons::{
    command::{Command, CommandConsumerGenerator},
    socket::{Socket, SocketCommand},
};
use log::{Level, debug, info};
use menu_back::{ToClient, ToServer};

struct MenuState {
    socket: Socket<ToClient, ToServer>,
}

#[derive(Debug)]
enum MenuCommand {
    Socket(SocketCommand<ToClient>),
}

impl Command<MenuState> for MenuCommand {
    fn apply(self, state: &mut MenuState) {
        debug!("{:?}", self);
        match self {
            MenuCommand::Socket(SocketCommand::Data(ToClient::Ping)) => {
                state.socket.send(ToServer::Pong);
            }
        }
    }
}

impl From<SocketCommand<ToClient>> for MenuCommand {
    fn from(value: SocketCommand<ToClient>) -> Self {
        MenuCommand::Socket(value)
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
    info!("Hello, world!");
    let mut consumer_generator: CommandConsumerGenerator<_, MenuCommand> =
        CommandConsumerGenerator::new();
    let state = MenuState {
        socket: Socket::new(consumer_generator.make_consumer(), "socket"),
    };
    consumer_generator.activate(state);
}
