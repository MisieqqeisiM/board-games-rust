use axum::{
    Form,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use tracing::info;

use crate::{ServerState, token::UserData};

impl FromRequestParts<ServerState> for UserData {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let token = jar
            .get("auth")
            .ok_or(Redirect::to("/login").into_response())?;
        state
            .auth_key
            .get_user_data(token.value_trimmed())
            .map_err(|_| Redirect::to("/login").into_response())
    }
}

#[derive(Deserialize)]
pub struct Login {
    username: String,
}

pub async fn login(
    State(state): State<ServerState>,
    jar: CookieJar,
    Form(user_data): Form<Login>,
) -> impl IntoResponse {
    info!("{}", user_data.username);
    let token = state.auth_key.get_token(UserData {
        username: user_data.username,
        id: rand::random(),
    });
    let Ok(token) = token else {
        return (StatusCode::FORBIDDEN, "Authentication failed").into_response();
    };
    let cookie = Cookie::new("auth", token);
    (jar.add(cookie), Redirect::to("/")).into_response()
}
