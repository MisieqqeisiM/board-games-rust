use frontend_commons::{
    command::{Command, CommandConsumerGenerator},
    list::{List, ListCommand},
    socket::{Socket, SocketCommand},
};
use log::{Level, debug, info};
use menu_back::{ToClient, ToServer};

struct MenuState {
    socket: Socket<ToClient, ToServer>,
    list: List,
}

#[derive(Debug)]
enum MenuCommand {
    Socket(SocketCommand<ToClient>),
    List(ListCommand),
}

impl Command<MenuState> for MenuCommand {
    fn apply(self, state: &mut MenuState) {
        debug!("{:?}", self);
        match self {
            MenuCommand::Socket(SocketCommand::Data(ToClient::ServerListEvent(ev))) => match ev {
                menu_back::server_list::ServerListEvent::Set(server_list) => {
                    for server in server_list.servers {
                        state.list.add_element(&server);
                    }
                }
                menu_back::server_list::ServerListEvent::Add(server) => {
                    state.list.add_element(&server);
                }
                menu_back::server_list::ServerListEvent::Remove(_) => {
                    todo!("Actually remove server from list")
                }
            },
            MenuCommand::List(ListCommand::Click(elem)) => {
                state.socket.send(ToServer::Pong);
            }
            _ => {}
        }
    }
}

impl From<SocketCommand<ToClient>> for MenuCommand {
    fn from(value: SocketCommand<ToClient>) -> Self {
        MenuCommand::Socket(value)
    }
}

impl From<ListCommand> for MenuCommand {
    fn from(value: ListCommand) -> Self {
        MenuCommand::List(value)
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
        list: List::new(consumer_generator.make_consumer()),
    };
    consumer_generator.activate(state);
}
