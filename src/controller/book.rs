use std::ops::Deref;

use crate::{auth::AuthenticatedUser, db::DataDb, exception::ApiError, time_dbg};
use actix_web::{get, web, Responder, Scope};
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Book {
    id: i32,
    pub title: String,
    timestamp: NaiveDateTime,
    uuid: String,
    has_cover: bool,
    last_modified: NaiveDateTime,
    pub path: String
}

#[get("")]
async fn books(data_db: DataDb) -> Result<impl Responder, ApiError> {
    let res: Vec<Book> = time_dbg!(sqlx::query_as(
        "SELECT * FROM books
         ORDER BY last_modified DESC
         LIMIT 100")
        .fetch_all(data_db.deref())
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?);
    Ok(web::Json(res))
}

#[get("/{id}")]
async fn book_detail(id: web::Path<u32>, user: AuthenticatedUser) -> impl Responder {
    format!("Book detail {} {}", id, user.username)
}

pub fn register() -> Scope {
    web::scope("/book")
        .service(books)
        .service(book_detail)
}