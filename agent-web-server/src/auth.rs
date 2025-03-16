/// JWT authentication.
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::SocketAddr,
};

use anyhow::{Context, anyhow};
use axum::{Json, extract::FromRequest, http::StatusCode};
use jwt_simple::prelude::*;
use serde::de::DeserializeOwned;

use crate::{
    indoc_info,
    states::{DATA_DIR, JWT_KEY, SERVER_CONFIG},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaim {
    pub uuid: String,
}

pub fn init_jwt_key() -> anyhow::Result<Vec<u8>> {
    let data_dir = DATA_DIR.get().unwrap();
    let jwt_key_path = data_dir.join("jwt_key");
    match OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(&jwt_key_path)
    {
        Ok(mut jwt_key_file) => {
            // not exist, generate key
            let key = HS256Key::generate();
            let key_bytes = key.to_bytes();
            jwt_key_file
                .write_all(&key_bytes)
                .with_context(|| "write jwt key")?;
            indoc_info!(
                "
                Newly created jwt key:
                {:?}
                ",
                key_bytes
            );
            Ok(key_bytes)
        }
        Err(_) => {
            // already exist, read key
            let mut jwt_key_file =
                File::open(&jwt_key_path).with_context(|| "open existing jwt key file.")?;
            let mut key_bytes = Vec::new();
            jwt_key_file
                .read_to_end(&mut key_bytes)
                .with_context(|| "read jwt key file")?;
            indoc_info!(
                "
                Existing jwt key:
                {:?}
                ",
                key_bytes
            );
            Ok(key_bytes)
        }
    }
}

pub fn gen_jwt(custom_claim: JwtClaim) -> String {
    let config = SERVER_CONFIG.get().unwrap();
    let key = JWT_KEY.get().unwrap();
    let claim =
        Claims::with_custom_claims(custom_claim, Duration::from_days(config.jwt_expire_days));
    match key.authenticate(claim) {
        Ok(s) => s,
        Err(_) => {
            unreachable!()
        }
    }
}

pub fn verify_jwt(token: &str) -> anyhow::Result<JwtClaim> {
    let key = JWT_KEY.get().unwrap();
    let claims = key
        .verify_token(token, None)
        .map_err(|_| anyhow!("invalid jwt"))?;
    Ok(claims.custom)
}

#[allow(unused)]
mod test {
    use super::*;

    #[test]
    fn test_jwt() {
        let key = HS256Key::generate();
        let key_str = String::from_utf8_lossy(&key.to_bytes()).to_string();
        println!("key: {key_str}");
        JWT_KEY.set(key.clone()).unwrap();
        let uuid = uuid::Uuid::new_v4().to_string();
        let custom_claim = JwtClaim { uuid };
        let claim = Claims::with_custom_claims(custom_claim, Duration::from_hours(2));
        let mut token = key.authenticate(claim).unwrap();
        println!("{token}");
        let res = verify_jwt(&token);
        assert!(res.is_ok());
        token.push('a');
        println!("{token}");
        let res = verify_jwt(&token);
        assert!(res.is_err());
    }
}

#[derive(Debug)]
pub struct AuthReq<T>
where
    T: DeserializeOwned,
{
    pub claim: JwtClaim,
    pub ip: Option<SocketAddr>,
    pub body: T,
}

impl<S, T> FromRequest<S> for AuthReq<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = req.headers().get("authorization").ok_or((
            // no authorization header key
            StatusCode::UNAUTHORIZED,
            "Missing authorization header.".to_string(),
        ))?;
        let auth_str = auth_header.to_str().map_err(|_| {
            (
                // authorization value not a valid string
                StatusCode::BAD_REQUEST,
                "Invalid authorization header.".to_string(),
            )
        })?;
        if !auth_str.starts_with("Bearer ") {
            return Err((
                // authorization value without specifying Bearer
                StatusCode::BAD_REQUEST,
                "Invalid authorization scheme.".to_string(),
            ));
        }
        let token = auth_str.trim_start_matches("Bearer ").trim();
        let claim = verify_jwt(token).map_err(|_| {
            (
                // invalid jwt
                StatusCode::UNAUTHORIZED,
                "Invalid or expired JWT.".to_string(),
            )
        })?;
        let ip = req
            .extensions()
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0);
        let Json(body) = Json::<T>::from_request(req, state).await.map_err(|err| {
            (
                // request body not a valid json
                StatusCode::BAD_REQUEST,
                format!("Failed to parse JSON body: {}", err),
            )
        })?;
        Ok(AuthReq { claim, ip, body })
    }
}
