use std::{
    marker::PhantomData,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, unbounded_channel},
    },
    time::{self, Instant},
};
use tracing::debug;

use crate::token::UserData;

pub struct Client<ToClient> {
    user_data: UserData,
    socket: SplitSink<WebSocket, Message>,
    to_client: PhantomData<ToClient>,
}

impl<ToClient> Client<ToClient> {
    pub fn get_id(&self) -> u64 {
        self.user_data.id
    }

    pub fn get_user_data(&self) -> UserData {
        self.user_data.clone()
    }
}

impl<ToClient> Client<ToClient>
where
    ToClient: Serialize,
{
    pub async fn ping(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_le_bytes();
        self.socket
            .send(Message::Ping(Bytes::from_owner(timestamp)))
            .await
            .unwrap()
    }

    pub async fn pong(&mut self, data: Bytes) {
        self.socket
            .send(Message::Pong(Bytes::from_owner(data)))
            .await
            .unwrap()
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

pub trait SocketHandler<ToClient, ToServer, Internal> {
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
    fn on_internal_message(
        &mut self,
        internal_message: Internal,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    fn on_ping(
        &mut self,
        client_id: u64,
        data: Bytes,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    fn on_pong(
        &mut self,
        client_id: u64,
        timestamp: u128,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    fn tick(&mut self) -> impl std::future::Future<Output = ()> + std::marker::Send;
}

pub struct SocketEndpoint<ToClient, ToServer, Internal> {
    message_sender: mpsc::UnboundedSender<ServerMessage<ToClient, ToServer, Internal>>,
    kill_sender: broadcast::Sender<()>,
    to_client: PhantomData<ToClient>,
    to_server: PhantomData<ToServer>,
}

impl<ToClient, ToServer, Internal> SocketEndpoint<ToClient, ToServer, Internal>
where
    ToClient: DeserializeOwned + Send + 'static,
    ToServer: DeserializeOwned + Send + 'static,
    Internal: Send + 'static,
{
    pub fn new(
        socket_handler: impl SocketHandler<ToClient, ToServer, Internal> + Send + 'static,
    ) -> Self {
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

    pub fn send_internal_message(&self, message: Internal) {
        self.message_sender
            .send(ServerMessage::InternalMessage(message))
            .unwrap();
    }

    pub fn handler(&self, ws: WebSocketUpgrade, user_data: UserData) -> Response {
        let message_sender = self.message_sender.clone();
        let kill_receiver = self.kill_sender.subscribe();
        ws.on_upgrade(move |socket| {
            on_upgrade::<ToClient, ToServer, Internal>(
                socket,
                user_data,
                message_sender,
                kill_receiver,
            )
        })
    }
}

impl<ToClient, ToServer, Internal> Drop for SocketEndpoint<ToClient, ToServer, Internal> {
    fn drop(&mut self) {
        self.kill_sender.send(()).unwrap();
    }
}

enum ServerMessage<ToClient, ToServer, Internal> {
    NewClient(Client<ToClient>),
    InternalMessage(Internal),
    Message { client_id: u64, message: ToServer },
    Disconnect { client_id: u64 },
    Ping { client_id: u64, data: Bytes },
    Pong { client_id: u64, timestamp: u128 },
}

async fn on_upgrade<ToClient, ToServer, Internal>(
    socket: WebSocket,
    user_data: UserData,
    message_sender: mpsc::UnboundedSender<ServerMessage<ToClient, ToServer, Internal>>,
    kill_receiver: broadcast::Receiver<()>,
) where
    ToClient: DeserializeOwned,
    ToServer: DeserializeOwned,
{
    let (to_client, from_client) = socket.split();
    let client = Client {
        user_data,
        socket: to_client,
        to_client: PhantomData,
    };
    socket_loop(message_sender, from_client, kill_receiver, client).await;
}

async fn socket_loop<ToClient, ToServer, Internal>(
    message_sender: mpsc::UnboundedSender<ServerMessage<ToClient, ToServer, Internal>>,
    mut from_client: SplitStream<WebSocket>,
    mut kill_receiver: broadcast::Receiver<()>,
    client: Client<ToClient>,
) -> Option<()>
where
    ToClient: DeserializeOwned,
    ToServer: DeserializeOwned,
{
    let client_id = client.get_id().to_owned();
    message_sender.send(ServerMessage::NewClient(client)).ok()?;
    loop {
        select! {
          Some(Ok(message)) = from_client.next() => {
            match message {
              Message::Binary(message) => {
                let Ok(message) = serde_cbor::from_slice(&message) else { break; };
                message_sender.send(ServerMessage::Message { client_id, message }).ok()?;
              },
              Message::Ping(data) => {
                message_sender.send(ServerMessage::Ping { client_id, data}).ok()?;
              },
              Message::Pong(data) => {
                let Ok(bytes) = data.as_ref().try_into() else {
                    break;
                };
                let timestamp = u128::from_le_bytes(bytes);
                message_sender.send(ServerMessage::Pong { client_id, timestamp}).ok()?;
              },
              Message::Close(_) => break,
              _ => ()
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
        .send(ServerMessage::Disconnect { client_id })
        .ok()?;
    Some(())
}

async fn pass_messages<ToClient, ToServer, Internal>(
    mut channel: UnboundedReceiver<ServerMessage<ToClient, ToServer, Internal>>,
    mut socket_handler: impl SocketHandler<ToClient, ToServer, Internal>,
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
                ServerMessage::Message{client_id,message} => socket_handler.on_message(client_id,message).await,
                ServerMessage::InternalMessage(internal_message) => socket_handler.on_internal_message(internal_message).await,
                ServerMessage::Disconnect{client_id} => socket_handler.on_disconnect(client_id).await,
                ServerMessage::Ping { client_id, data } => socket_handler.on_ping(client_id, data).await,
                ServerMessage::Pong { client_id, timestamp } => socket_handler.on_pong(client_id, timestamp).await,
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
