use std::{fs::File, io::Read, ops::Deref, path::Path};

use crate::{
    auth::AuthenticatedUser, db::DataDb, exception::ApiError, time_dbg, DATA_DB,
};
use actix_web::{get, web, Responder, Scope};
use chrono::NaiveDateTime;
use reqwest::{multipart::Part, Client};
use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct Book {
    id: i32,
    pub title: String,
    timestamp: NaiveDateTime,
    uuid: String,
    has_cover: bool,
    last_modified: NaiveDateTime,
    pub path: String,
}

const BACKUP_URL: &str = "https://cyber.mazhangjing.com/cyber/books/updating-with-calibre-db";

#[get("")]
async fn books(data_db: DataDb) -> Result<impl Responder, ApiError> {
    let res: Vec<Book> = time_dbg!(sqlx::query_as(
        "SELECT * FROM books
         ORDER BY last_modified DESC
         LIMIT 100"
    )
    .fetch_all(data_db.deref())
    .await
    .map_err(|e| ApiError::DbError(e.to_string()))?);
    Ok(web::Json(res))
}

#[get("/sync/{token}")]
async fn sync_book_db(token: web::Path<String>) -> Result<impl Responder, ApiError> {
    let db_file = Path::new(&DATA_DB);
    let mut file = File::open(db_file).map_err(|e| ApiError::NotFoundFile(e.to_string()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| ApiError::NotFoundFile(e.to_string()))?;
    let response = Client::new()
        .post(BACKUP_URL)
        .header("Authorization", format!("Basic {}", token.deref()))
        .multipart(
            reqwest::multipart::Form::new()
                .part(
                    "file",
                    Part::bytes(buffer)
                        .file_name("metadata.db")
                        .mime_str("application/octet-stream")
                        .map_err(|e| ApiError::NotFoundFile(e.to_string()))?,
                )
                .text("filename", "metadata.db")
                .text("truncate", "true"),
        )
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;
    Ok(body
        .customize()
        .append_header(("Content-Type", "application/json")))
}

#[get("/{id}")]
async fn book_detail(id: web::Path<u32>, user: AuthenticatedUser) -> impl Responder {
    format!("Book detail {} {}", id, user.username)
}

pub fn register() -> Scope {
    web::scope("/book")
    .service(sync_book_db)
    .service(books).service(book_detail)
}
