use std::{marker::PhantomData, time::Duration};

use axum::{
    body::Bytes,
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, unbounded_channel},
    },
    time::{self, Instant},
};

pub struct Client<ToClient> {
    id: u64,
    socket: SplitSink<WebSocket, Message>,
    to_client: PhantomData<ToClient>,
}

impl<ToClient> Client<ToClient>
where
    ToClient: Serialize,
{
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub async fn send(&mut self, message: ToClient) {
        let _ = self
            .socket
            .send(Message::Binary(Bytes::from(
                serde_cbor::to_vec(&message).unwrap(),
            )))
            .await;
    }
}

pub trait SocketHandler<ToClient, ToServer> {
    fn on_connect(
        &mut self,
        client: Client<ToClient>,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    fn on_message(
        &mut self,
        client_id: u64,
        message: ToServer,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    fn on_disconnect(
        &mut self,
        client_id: u64,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    fn tick(&mut self) -> impl std::future::Future<Output = ()> + std::marker::Send;
}

pub struct SocketEndpoint<ToClient, ToServer> {
    message_sender: mpsc::UnboundedSender<ServerMessage<ToClient, ToServer>>,
    kill_sender: broadcast::Sender<()>,
    to_client: PhantomData<ToClient>,
    to_server: PhantomData<ToServer>,
}

impl<ToClient, ToServer> SocketEndpoint<ToClient, ToServer>
where
    ToClient: DeserializeOwned + Send + 'static,
    ToServer: DeserializeOwned + Send + 'static,
{
    pub fn new(socket_handler: impl SocketHandler<ToClient, ToServer> + Send + 'static) -> Self {
        let (message_sender, message_receiver) = unbounded_channel();
        let (kill_sender, _) = broadcast::channel(1);
        tokio::spawn(pass_messages(
            message_receiver,
            socket_handler,
            kill_sender.subscribe(),
        ));
        let state = SocketEndpoint {
            message_sender,
            kill_sender,
            to_client: PhantomData,
            to_server: PhantomData,
        };
        state
    }

    pub fn handler(&self, ws: WebSocketUpgrade) -> Response {
        let message_sender = self.message_sender.clone();
        let kill_receiver = self.kill_sender.subscribe();
        ws.on_upgrade(move |socket| {
            on_upgrade::<ToClient, ToServer>(socket, message_sender, kill_receiver)
        })
    }
}

impl<ToClient, ToServer> Drop for SocketEndpoint<ToClient, ToServer> {
    fn drop(&mut self) {
        self.kill_sender.send(()).unwrap();
    }
}

enum ServerMessage<ToClient, ToServer> {
    NewClient(Client<ToClient>),
    Message { client_id: u64, message: ToServer },
    Disconnect { client_id: u64 },
}

async fn on_upgrade<ToClient, ToServer>(
    socket: WebSocket,
    message_sender: mpsc::UnboundedSender<ServerMessage<ToClient, ToServer>>,
    kill_receiver: broadcast::Receiver<()>,
) where
    ToClient: DeserializeOwned,
    ToServer: DeserializeOwned,
{
    let id = rand::random::<u64>();
    let (to_client, from_client) = socket.split();
    let client = Client {
        id,
        socket: to_client,
        to_client: PhantomData,
    };
    socket_loop(message_sender, from_client, kill_receiver, client).await;
}

async fn socket_loop<ToClient, ToServer>(
    message_sender: mpsc::UnboundedSender<ServerMessage<ToClient, ToServer>>,
    mut from_client: SplitStream<WebSocket>,
    mut kill_receiver: broadcast::Receiver<()>,
    client: Client<ToClient>,
) -> Option<()>
where
    ToClient: DeserializeOwned,
    ToServer: DeserializeOwned,
{
    let id = client.id.to_owned();
    message_sender.send(ServerMessage::NewClient(client)).ok()?;
    loop {
        select! {
          Some(Ok(message)) = from_client.next() => {
            match message {
              Message::Binary(message) => {
                let Ok(message) = serde_cbor::from_slice(&message) else { break; };
                message_sender.send(ServerMessage::Message { client_id: id, message }).ok()?;
              },
              Message::Close(_) => break,
              _ => continue
            }
          },
          _ = kill_receiver.recv() => {
            break;
          },
          else => {
            break;
          }
        }
    }
    message_sender
        .send(ServerMessage::Disconnect { client_id: id })
        .ok()?;
    Some(())
}

async fn pass_messages<ToClient, ToServer>(
    mut channel: UnboundedReceiver<ServerMessage<ToClient, ToServer>>,
    mut socket_handler: impl SocketHandler<ToClient, ToServer>,
    mut kill_receiver: broadcast::Receiver<()>,
) {
    let mut interval = time::interval_at(
        Instant::now() + Duration::from_secs(5),
        Duration::from_secs(5),
    );
    loop {
        select! {
          Some(message) = channel.recv() => {
            match message {
              ServerMessage::NewClient(client) => socket_handler.on_connect(client).await,
              ServerMessage::Message { client_id, message } =>
                socket_handler.on_message(client_id, message).await,
              ServerMessage::Disconnect { client_id } => socket_handler.on_disconnect(client_id).await,
            };
          },
          _ = interval.tick() => {
            socket_handler.tick().await;
          },
          _ = kill_receiver.recv() => {
            break;
          },
          else => {
            break;
          }
        }
    }
}
