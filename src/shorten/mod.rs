mod shorten_service;

use crate::shorten::shorten_service::{create_shorten_link, get_shorten_link};
use anyhow::Result;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::sync::Arc;
use thiserror::Error;
use tracing::log::{info, log, logger};

#[derive(Debug)]
struct AppState {
    pool: Pool<Postgres>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    user: String,
    password: String,
    host: String,
    dbname: String,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("data error")]
    DbError(#[from] sqlx::error),

    #[error("general error: {0}")]
    AnyError(#[from] anyhow::Error),
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
        let url = format!(
            "DATABASE_URL=postgres://{}:{}@{}/{}",
            self.user, self.password, self.host, self.dbname
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
            .await
            .unwrap();

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