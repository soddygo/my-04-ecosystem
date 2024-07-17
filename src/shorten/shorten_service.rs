use crate::shorten::{AppError, AppState};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::Deserialize;
use sqlx::{Decode, Error, FromRow};
use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use sqlx::postgres::PgQueryResult;

#[derive(FromRow, Deserialize,Decode)]
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
) -> Result<impl IntoResponse,AppError >{
    // todo!()

    let url = payload["url"].as_str().unwrap();

    //查询数据库有无数据记录，根据url查找
    let one_data: Option<Shorten> = sqlx::query_as("SELECT * FROM shorten WHERE url = $1")
        .bind(url)
        .fetch_optional(&state.pool)
        .await?;

    match one_data {
        Some(data) => {
            //更新数据库
            let update_id = data.id;
          sqlx::query("UPDATE shorten SET url = $1 WHERE id = $2")
                .bind(url)
                .bind(update_id)
                .execute(&state.pool)
                .await?;
        }
        None => {
            //插入数据库
            sqlx::query("INSERT INTO shorten (url) VALUES ($1) RETURNING *")
                .bind(url)
                .execute(&state.pool)
                .await?;
        }
    }

   Ok( "Hello, World!")
}

pub(crate) async fn get_shorten_link(Path(id): Path<String>) -> impl IntoResponse {
    // todo!()
    "Hello, World!"
}
