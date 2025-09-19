mod auth;
mod menu_server;
mod socket_endpoint;

use std::{collections::BTreeMap, error::Error, sync::Arc};

use axum::{
    Form, Router,
    extract::{Request, State, WebSocketUpgrade},
    http::{HeaderValue, StatusCode, header::SET_COOKIE},
    middleware::{Next, from_fn, from_fn_with_state},
    response::{ErrorResponse, IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use axum_macros::debug_handler;
use hmac::{Hmac, digest::KeyInit};
use jwt::{Header, SignWithKey};
use menu_back::{ToClient, ToServer};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing::info;

use crate::{
    auth::{Key, UserData},
    menu_server::Menu,
    socket_endpoint::SocketEndpoint,
};

#[derive(Clone)]
struct ServerState {
    auth_key: Key,
    menu: Arc<Mutex<SocketEndpoint<ToClient, ToServer>>>,
}

async fn ws(ws: WebSocketUpgrade, State(state): State<ServerState>) -> Response {
    info!("Websocket connection");
    let menu = state.menu.lock().await;
    menu.handler(ws)
}

#[derive(Deserialize)]
struct Login {
    username: String,
}

async fn login(
    State(state): State<ServerState>,
    jar: CookieJar,
    Form(user_data): Form<Login>,
) -> impl IntoResponse {
    info!("{}", user_data.username);
    let token = state.auth_key.get_token(UserData {
        username: user_data.username,
    });
    let Ok(token) = token else {
        return (StatusCode::FORBIDDEN, "Authentication failed").into_response();
    };
    let cookie = Cookie::new("auth", token);
    (jar.add(cookie), Redirect::to("/")).into_response()
}

async fn auth_middleware(
    State(state): State<ServerState>,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {
    let Some(token) = jar.get("auth") else {
        return Redirect::to("/login").into_response();
    };
    let Ok(data) = state.auth_key.get_user_data(token.value_trimmed()) else {
        return Redirect::to("/login").into_response();
    };

    info!("{}", data.username);

    next.run(request).await
}

async fn redirect_to_menu() -> Redirect {
    Redirect::to("/menu")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("Starting server");

    let state = ServerState {
        menu: Arc::new(Mutex::new(SocketEndpoint::new(Menu::new()))),
        auth_key: Key::new("test-key".to_owned()).unwrap(),
    };

    let app = Router::new()
        .route("/", get(redirect_to_menu))
        .route("/socket", get(ws))
        .nest_service("/menu", ServeDir::new("../menu_front/dist"))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware))
        .nest_service("/login", ServeDir::new("../login/dist"))
        .route("/login_handler", post(login))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
