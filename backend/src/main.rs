mod auth;
mod event_store;
mod menu_server;
mod socket_endpoint;
mod test_server;
mod token;

use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    extract::{Path, Request, State, WebSocketUpgrade},
    middleware::{Next, from_fn_with_state},
    response::{Redirect, Response},
    routing::{get, post},
};
use menu_back::{ToClient, ToServer};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

use crate::{
    auth::login,
    menu_server::{Menu, MenuMessage},
    socket_endpoint::SocketEndpoint,
    test_server::Test,
    token::{Key, UserData},
};

#[derive(Clone)]
struct ServerState {
    auth_key: Key,
    menu: Arc<Mutex<SocketEndpoint<ToClient, ToServer, MenuMessage>>>,
    test_rooms:
        Arc<Mutex<HashMap<String, SocketEndpoint<test_back::ToClient, test_back::ToServer, ()>>>>,
}

async fn ws(
    ws: WebSocketUpgrade,
    user_data: UserData,
    State(state): State<ServerState>,
) -> Response {
    info!(
        "Websocket connection from {} {}",
        user_data.id, user_data.username
    );
    let menu = state.menu.lock().await;
    menu.handler(ws, user_data)
}

async fn test_ws(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    user_data: UserData,
    State(state): State<ServerState>,
) -> Response {
    let mut rooms = state.test_rooms.lock().await;
    let menu = state.menu.lock().await;
    if rooms.contains_key(&room_id) {
        let room = rooms.get(&room_id).unwrap();
        room.handler(ws, user_data)
    } else {
        let test = Test::new(room_id.clone()).await;
        let room = SocketEndpoint::new(test);
        let handler = room.handler(ws, user_data);
        rooms.insert(room_id.clone(), room);
        menu.send_internal_message(MenuMessage::ServerCreated(room_id.clone()));
        handler
    }
}

async fn auth_middleware(_user_dat: UserData, request: Request, next: Next) -> Response {
    next.run(request).await
}

async fn redirect_to_menu() -> Redirect {
    Redirect::to("/menu")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("Starting server");

    let test_rooms = Arc::new(Mutex::new(HashMap::new()));
    let menu = Arc::new(Mutex::new(SocketEndpoint::new(Menu::new())));

    let state = ServerState {
        menu,
        test_rooms,
        auth_key: Key::new("test-key".to_owned()).unwrap(),
    };

    let app = Router::new()
        .route("/", get(redirect_to_menu))
        .nest_service("/menu", ServeDir::new("../menu_front/dist"))
        .route("/menu/socket", get(ws))
        .nest_service("/test/static", ServeDir::new("../test_front/dist"))
        .nest_service(
            "/test/{room_id}",
            ServeFile::new("../test_front/dist/index.html"),
        )
        .route("/test/{room_id}/socket", get(test_ws))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware))
        .nest_service("/login", ServeDir::new("../login/dist"))
        .route("/login_handler", post(login))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
