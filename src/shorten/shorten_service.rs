use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, FromRow};

use crate::shorten::{AppError, AppState};

#[derive(FromRow, Deserialize, Serialize)]
pub(crate) struct Shorten {
    #[sqlx(default)]
    id: i32,
    #[sqlx(default)]
    url: String,
    #[sqlx(default)]
    created_at: DateTime<Utc>,
}

/// create shorten link by
pub(crate) async fn create_shorten_link(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Shorten>, AppError> {
    let url = payload["url"].as_str().unwrap();

    //查询数据库有无数据记录，根据url查找
    let one_data: Option<Shorten> = sqlx::query_as("SELECT * FROM shorten WHERE url = $1")
        .bind(url)
        .fetch_optional(&state.pool)
        .await?;

    let shorten_one: Shorten = match one_data {
        Some(data) => {
            //更新数据库
            let update_id = data.id;
            sqlx::query_as("UPDATE shorten SET url = $1 WHERE id = $2  RETURNING *")
                .bind(url)
                .bind(update_id)
                .fetch_one(&state.pool)
                .await?
        }
        None => {
            //插入数据库
            sqlx::query_as("INSERT INTO shorten (url) VALUES ($1) RETURNING *")
                .bind(url)
                .fetch_one(&state.pool)
                .await?
        }
    };

    let json = Json(shorten_one);
    return Ok(json);
}

pub(crate) async fn get_shorten_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Shorten>, AppError> {
    let shorten: Option<Shorten> = sqlx::query_as("SELECT * FROM shorten WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;


    match shorten {
        Some(data) => Ok(Json(data)),
        None => Err(AppError::AnyError(anyhow!("Resource not found"))),
    }
}
