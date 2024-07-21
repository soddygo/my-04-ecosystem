use std::sync::Arc;

use crate::shorten::{AppError, AppState};
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, FromRow};
use tracing::info;

#[derive(FromRow, Deserialize, Serialize)]
pub(crate) struct Shorten {
    #[sqlx(default)]
    id: i64,
    #[sqlx(default)]
    url: String,
    #[sqlx(default)]
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ShortenReq {
    url: String,
}

/// create shorten link by
pub(crate) async fn create_shorten_link(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ShortenReq>,
) -> Result<Json<Shorten>, AppError> {
    let url = payload.url;

    //查询数据库有无数据记录，根据url查找
    let one_data: Option<Shorten> = sqlx::query_as("SELECT * FROM shorten WHERE url = $1")
        .bind(&url)
        .fetch_optional(&state.pool)
        .await?;

    let shorten_one: Shorten = match one_data {
        Some(data) => {
            //更新数据库
            let update_id = data.id;
            sqlx::query_as("UPDATE shorten SET url = $1 WHERE id = $2  RETURNING *")
                .bind(&url)
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
    Ok(json)
}

pub(crate) async fn get_shorten_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("get shorten link by id: {}", id);

    let int_number = id.parse::<i64>()?;
    let shorten: Option<Shorten> = sqlx::query_as("SELECT * FROM shorten WHERE id = $1")
        .bind(int_number)
        .fetch_optional(&state.pool)
        .await?;

    match shorten {
        Some(data) => {
            let redirect = Redirect::to(&data.url);
            info!("redirect to {}", data.url);
            Ok(redirect)
        }
        None => Err(AppError::AnyError(anyhow!("Resource not found"))),
    }
}
