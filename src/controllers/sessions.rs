use super::*;

use chrono::{DateTime, Local, TimeZone};
use serde::Serialize;
use sqlx::MySql;
use tide::{Body, Request};
use tide_sqlx::SQLxRequestExt;

pub async fn auth_by_minute(req: Request<State>) -> tide::Result<Body> {
    let mut db_pool = req.sqlx_conn::<MySql>().await;
    let connection = db_pool.acquire().await?;
    let rows = handlers::sessions::recent_auth_attempts(connection, 6).await?;

    Body::from_json(&rows)
}
