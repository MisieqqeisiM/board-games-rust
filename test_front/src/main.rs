use std::collections::HashMap;

use frontend_commons::{
    client_info::ClientInfo,
    command::{Command, CommandConsumerGenerator},
    list::{List, ListCommand},
    socket::{Socket, SocketCommand},
};
use log::{Level, debug, info};
use test_back::{ToClient, ToServer};

struct TestState {
    socket: Socket<ToClient, ToServer>,
    clients: HashMap<u64, ClientInfo>,
}

#[derive(Debug)]
enum TestCommand {
    Socket(SocketCommand<ToClient>),
}

impl Command<TestState> for TestCommand {
    fn apply(self, state: &mut TestState) {
        debug!("{:?}", self);
        match self {
            TestCommand::Socket(SocketCommand::Data(ToClient::ClientListMessage(message))) => {
                // debug!("{:?}", message);
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

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
    info!("Hello, world!");
    let mut consumer_generator: CommandConsumerGenerator<_, TestCommand> =
        CommandConsumerGenerator::new();
    let state = TestState {
        socket: Socket::new(consumer_generator.make_consumer(), "socket"),
        clients: HashMap::new(),
    };
    consumer_generator.activate(state);
}
