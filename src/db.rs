use std::{ops::Deref, sync::Arc};

use actix_web::{http::Error, web, FromRequest};
use futures::future::{ready, Ready};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::{WEB_DB, DATA_DB};

#[derive(Clone)]
pub struct DbState {
    pub data_db: SqlitePool,
    pub web_db: SqlitePool,
}

impl DbState {
    pub async fn connect() -> Arc<DbState> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(format!("sqlite:{DATA_DB}").as_str())
            .await
            .expect(format!("connect db {} failed", DATA_DB).as_str());
        let pool2 = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(format!("sqlite:{WEB_DB}").as_str())
            .await
            .expect(format!("connect db {} failed", WEB_DB).as_str());
        let db_state = DbState {
            data_db: pool,
            web_db: pool2,
        };
        Arc::new(db_state)
    }
}

pub struct DataDb(pub SqlitePool);

pub struct WebDb(pub SqlitePool);

impl Deref for DataDb {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for WebDb {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for DataDb {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let data = req.app_data::<web::Data<Arc<DbState>>>().unwrap();
        ready(Ok(DataDb(data.data_db.clone())))
    }
}

impl FromRequest for WebDb {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let data = req.app_data::<web::Data<Arc<DbState>>>().unwrap();
        ready(Ok(WebDb(data.web_db.clone())))
    }
}
