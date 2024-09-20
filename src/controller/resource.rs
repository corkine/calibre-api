use actix_web::{get, web, Responder, Scope};

#[get("/{hash}")]
async fn image(hash: web::Path<String>) -> impl Responder {
    format!("Image Detail {}", hash)
}

pub fn register() -> Scope {
    web::scope("/resource")
    .service(
        actix_files::Files::new("/", "../calibre-data")
        .use_etag(true)
    )
}