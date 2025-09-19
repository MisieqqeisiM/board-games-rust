mod menu_server;
mod socket_endpoint;

use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
    routing::get,
};
use menu_back::{ToClient, ToServer};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing::info;

use crate::{menu_server::Menu, socket_endpoint::SocketEndpoint};

struct ServerState {
    menu: SocketEndpoint<ToClient, ToServer>,
}

async fn ws(ws: WebSocketUpgrade, State(state): State<Arc<Mutex<ServerState>>>) -> Response {
    info!("Websocket connection");
    let state = state.lock().await;
    state.menu.handler(ws)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("Starting server");

    let state = ServerState {
        menu: SocketEndpoint::new(Menu::new()),
    };

    let app = Router::new()
        .route("/socket", get(ws))
        .fallback_service(ServeDir::new("../menu_front/dist"))
        .with_state(Arc::new(Mutex::new(state)));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
