mod shorten_service;

use crate::shorten::shorten_service::{create_shorten_link, get_shorten_link};
use anyhow::Result;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{http::status::StatusCode, Router};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::num::ParseIntError;
use std::sync::Arc;
use thiserror::Error;
use tracing::log::{info, log, logger};

#[derive(Debug)]
struct AppState {
    pool: Pool<Postgres>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DataBase {
    user: String,
    password: String,
    host: String,
    dbname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    //重命名为“database”
    #[serde(rename = "database")]
    data_base: DataBase,
}

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("data error:{0}")]
    DbError(#[from] sqlx::Error),

    #[error("general error: {0}")]
    AnyError(#[from] anyhow::Error),

    #[error("parse int error:{0}")]
    ParseError(#[from] ParseIntError),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            AppError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status_code,
            format!("Something went wrong: {}", self.to_string()),
        )
            .into_response()
    }
}

impl Config {
    //get db url
    fn get_db_url(self) -> String {
        let database = self.data_base;
        let url = format!(
            "postgres://{}:{}@{}/{}",
            database.user, database.password, database.host, database.dbname
        );
        info!("db url:{}", url);
        return url;
    }
}

impl AppState {
    async fn try_new() -> Result<Self> {
        // 从配置文件中，读取数据库配置
        let config_file = fs::read("src/config.yml")?;
        let config: Config = serde_yaml::from_slice(&config_file)?;

        let db_url = config.get_db_url();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let app = Self { pool };
        Ok(app)
    }
}

///init shorten router
pub(crate) async fn init_shorten_router() -> anyhow::Result<Router> {
    let app_state = AppState::try_new().await?;
    let arc_app_state = Arc::new(app_state);
    let shorten_router = Router::new()
        .route("/:id", get(get_shorten_link))
        .route("/create", post(create_shorten_link))
        .with_state(arc_app_state);

    Ok(shorten_router)
}
