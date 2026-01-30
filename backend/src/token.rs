use hmac::{Hmac, Mac, digest::InvalidLength};
use jwt::{Error, Header, SignWithKey, Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Clone)]
pub struct Key(Hmac<Sha256>);

#[derive(Debug)]
pub enum AuthError {
    KeyError(InvalidLength),
    SigningError(Error),
    InvalidToken(Error),
}

#[derive(Serialize, Deserialize)]
struct AuthenticationToken {
    sub: String,
    id: u64,
}

#[derive(Clone)]
pub struct UserData {
    pub id: u64,
    pub username: String,
}

impl Key {
    pub fn new(secret: String) -> Result<Self, AuthError> {
        let internal =
            Hmac::new_from_slice(secret.as_bytes()).map_err(|e| AuthError::KeyError(e))?;
        Ok(Key(internal))
    }

    pub fn get_token(&self, data: UserData) -> Result<String, AuthError> {
        let claims = AuthenticationToken {
            sub: data.username,
            id: data.id,
        };
        claims
            .sign_with_key(&self.0)
            .map_err(|e| AuthError::SigningError(e))
    }

    pub fn get_user_data(&self, token: &str) -> Result<UserData, AuthError> {
        let token: Token<Header, AuthenticationToken, _> = token
            .verify_with_key(&self.0)
            .map_err(|e| AuthError::InvalidToken(e))?;
        let claims = token.claims();
        Ok(UserData {
            username: claims.sub.to_owned(),
            id: claims.id,
        })
    }
}
