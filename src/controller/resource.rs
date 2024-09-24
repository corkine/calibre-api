use std::path::Path;

use actix_files::NamedFile;

use actix_web::{get, web, Responder, Scope};

use crate::{controller::book::Book, db::DataDb, exception::ApiError, time_dbg, DATA_DIR};

#[get("/cover/{hash}")]
async fn cover(hash: web::Path<String>, db: DataDb) -> Result<impl Responder, ApiError> {
    let res: Option<Book> = time_dbg!(sqlx::query_as("select * from books where uuid = ?")
        .bind(hash.as_str())
        .fetch_optional(&db.0)
        .await
        .map_err(|e| ApiError::DbError(e.to_string()))?);
    match res {
        Some(book) => {
            let path = format!("{}/{}/cover.jpg", &DATA_DIR, &book.path);
            let cover_path = Path::new(&path);
            let file = NamedFile::open(cover_path).map_err(|e| ApiError::NotFoundFile(e.to_string()))?;
            Ok(file.use_etag(true))
        }
        None => Err(ApiError::NotFound),
    }
}

pub fn register() -> Scope {
    web::scope("/resource")
    .service(cover)
    .service(actix_files::Files::new("/", "../calibre-data").use_etag(true))
}
