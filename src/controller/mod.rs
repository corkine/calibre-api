use actix_web::{get, web, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::json;

use crate::auth::validator;

mod book;
mod resource;

#[get("/")]
async fn welcome() -> impl Responder {
    json!({"message":"Welcome to the Book API"})
        .to_string()
        .customize()
        .insert_header(("Content-Type", "application/json"))
}

pub fn config_controller(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::with_fn(validator);
    cfg.service(welcome)
        .service(book::register().wrap(auth.clone()))
        .service(resource::register().wrap(auth.clone()));
}
