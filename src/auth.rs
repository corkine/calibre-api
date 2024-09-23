use actix_web::dev::Payload;
use actix_web::dev::ServiceRequest;
use actix_web::web;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::authorization::Basic;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use crate::db::DbState;
use crate::encrypt;
use crate::exception::ApiError;
use crate::time_dbg;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let user = req.extensions().get::<AuthenticatedUser>().cloned();
        match user {
            Some(u) => ready(Ok(u)),
            None => ready(Err(Error::from(actix_web::error::ErrorUnauthorized(
                "Unauthorized",
            )))),
        }
    }
}

struct AuthInfo {
    token: String,
    expires_at: Instant,
}

struct AuthCache {
    cache: Mutex<HashMap<String, AuthInfo>>,
}

impl AuthCache {
    fn new() -> Self {
        AuthCache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn set(&self, key: String, token: String, ttl: Duration) {
        let mut cache = self.cache.lock().unwrap();
        let expires_at = Instant::now() + ttl;
        cache.insert(key, AuthInfo { token, expires_at });
    }

    fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(auth_info) = cache.get(key) {
            if Instant::now() < auth_info.expires_at {
                Some(auth_info.token.clone())
            } else {
                cache.remove(key);
                None
            }
        } else {
            None
        }
    }
}

lazy_static::lazy_static! {
    static ref AUTH_CACHE: AuthCache = AuthCache::new();
}

use base64::decode;

fn extract_token_auth(req: &ServiceRequest) -> Option<BasicAuth> {
    let token = req
        .query_string()
        .split('&')
        .find(|&param| param.starts_with("token="))
        .and_then(|param| param.strip_prefix("token="))?;

    let decoded = match decode(token) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    let credentials = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let mut parts = credentials.splitn(2, ':');
    let username = parts.next()?;
    let password = parts.next();

    Some(
        Basic::new(
            username.to_string(),
            Some(password.unwrap_or("").to_string()),
        )
        .into(),
    )
}

pub async fn validator(
    req: ServiceRequest,
    credentials: Option<BasicAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Some(credentials) = credentials.or(extract_token_auth(&req)) {
        let user = credentials.user_id();
        let pass = credentials.password().unwrap_or("");
        if AUTH_CACHE
            .get(user)
            .map(|k| k == pass)
            .or(Some(false))
            .unwrap()
        {
            req.extensions_mut().insert(AuthenticatedUser {
                username: user.to_string(),
            });
            return Ok::<ServiceRequest, (Error, ServiceRequest)>(req);
        }
        let db = &req.app_data::<web::Data<Arc<DbState>>>().unwrap().web_db;
        match time_dbg!(
            sqlx::query_as::<_, (String,)>("SELECT password FROM user WHERE name = ?")
                .bind(user)
                .fetch_one(db)
                .await
        ) {
            Ok(pass_db) => {
                if time_dbg!(check_password_hash(user, pass_db.0.as_str(), pass)) {
                    req.extensions_mut().insert(AuthenticatedUser {
                        username: user.to_string(),
                    });
                    return Ok::<ServiceRequest, (Error, ServiceRequest)>(req);
                }
            }
            Err(_) => (),
        }
    }
    Err((ApiError::Unauthorized.into(), req))
}

fn check_password_hash(user: &str, pwhash: &str, password: &str) -> bool {
    let res = encrypt::check_password_hash(pwhash, password);
    if res {
        AUTH_CACHE.set(
            user.to_string(),
            password.to_string(),
            Duration::from_secs(60 * 60 * 24 * 7),
        );
    }
    res
}
