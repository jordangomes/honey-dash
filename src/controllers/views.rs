use super::*;

use chrono::{DateTime, Local, TimeZone};
use serde::Serialize;
use sqlx::MySql;
use tide::Request;
use tide_sqlx::SQLxRequestExt;

#[derive(Clone, Serialize)]
struct IPAggregateView {
    ip: String,
    first_seen: String,
    last_seen: String,
    sessions: i64,
    auth_attempts: i64,
    commands: i64,
    downloads: i64,
}

pub async fn index(req: Request<State>) -> tide::Result {
    let tera = req.state().tera.clone();
    let mut db_pool = req.sqlx_conn::<MySql>().await;
    let connection = db_pool.acquire().await?;
    let rows = handlers::sessions::ip_aggregate(connection, 50).await?;

    let mut converted_rows: Vec<IPAggregateView> = Vec::new();

    for row in rows {
        let first_seen_raw = row.first_seen.unwrap_or_default();
        let last_seen_raw = row.last_seen.unwrap_or_default();

        let first_seen_local: DateTime<Local> = Local::from_utc_datetime(&Local, &first_seen_raw);
        let last_seen_local: DateTime<Local> = Local::from_utc_datetime(&Local, &last_seen_raw);
        let now = chrono::offset::Local::now();

        let formatter = timeago::Formatter::new();
        let first_seen = formatter.convert_chrono(first_seen_local, now);
        let last_seen = formatter.convert_chrono(last_seen_local, now);

        converted_rows.push(IPAggregateView {
            ip: row.ip,
            first_seen,
            last_seen,
            sessions: row.sessions,
            auth_attempts: row.auth_attempts,
            commands: row.commands,
            downloads: row.downloads,
        });
    }

    tera.render_response(
        "index.html",
        &context! {
            "title" => String::from("Test Title"),
            "ip_aggregate" => converted_rows
        },
    )
}

#[derive(Serialize, Debug)]
pub struct AuthView {
    pub id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub success: i8,
    pub client: Option<String>,
    pub time: String,
}

pub async fn ip_details(req: Request<State>) -> tide::Result {
    let tera = req.state().tera.clone();
    let mut db_pool = req.sqlx_conn::<MySql>().await;
    let connection = db_pool.acquire().await?;

    let ip_param = req.param("ip").unwrap_or("0.0.0.0");
    let ip = ip_param.to_string();
    let rows = handlers::sessions::ip_auth_attempts(connection, ip).await?;

    let mut converted_rows: Vec<AuthView> = Vec::new();
    for row in rows {
        let timestamp_local: DateTime<Local> = Local::from_utc_datetime(&Local, &row.timestamp);
        let now = chrono::offset::Local::now();

        let mut formatter = timeago::Formatter::new();
        formatter.num_items(2);
        let time = formatter.convert_chrono(timestamp_local, now);

        let success: i8 = match row.success {
            None => 2,
            Some(n) => {
                if n == 0 {
                    0
                } else {
                    1
                }
            }
        };

        converted_rows.push(AuthView {
            id: row.id,
            username: row.username,
            password: row.password,
            success,
            client: row.client,
            time,
        });
    }

    tera.render_response(
        "ip_details.html",
        &context! {
            "title" => format!("{} - honeydash", &ip_param),
            "ip" => &ip_param,
            "auth_attempts" => converted_rows
        },
    )
}
