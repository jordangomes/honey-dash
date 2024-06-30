use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlConnection, Type};

#[derive(PartialEq, Debug, Type, Serialize, Deserialize)]
pub struct Session {
    id: String,
    starttime: NaiveDateTime,
    endtime: Option<NaiveDateTime>,
    sensor: i32,
    ip: String,
    termsize: Option<String>,
    client: Option<i32>,
}

#[derive(PartialEq, Debug, Type, Serialize, Deserialize)]
pub struct IPAggregate {
    pub ip: String,
    pub first_seen: Option<NaiveDateTime>,
    pub last_seen: Option<NaiveDateTime>,
    pub sessions: i64,
    pub auth_attempts: i64,
    pub commands: i64,
    pub downloads: i64,
}

#[derive(Debug)]
struct Auth {
    id: u64,
    session: Session,
    success: bool,
    username: String,
    password: String,
    timestamp: NaiveDateTime,
}

pub async fn ip_aggregate(
    connection: &mut MySqlConnection,
    limit: i64,
) -> tide::Result<Vec<IPAggregate>> {
    let query = sqlx::query_as!(IPAggregate, " select ip,MIN(starttime) AS first_seen,MAX(starttime) AS last_seen, COUNT(sessions.id) AS sessions, COUNT(auth.id) AS auth_attempts, COUNT(input.id) AS commands, COUNT(downloads.id) AS downloads FROM sessions LEFT JOIN auth ON sessions.id=auth.session LEFT JOIN input ON sessions.id=input.session LEFT JOIN downloads ON sessions.id=downloads.session GROUP BY ip ORDER BY last_seen DESC LIMIT ?;", limit)
        .fetch_all(connection)
        .await
        .map_err(|e| tide::Error::new(409, e))?;

    Ok(query)
}

pub async fn list_recent(connection: &mut MySqlConnection) -> tide::Result<Vec<Session>> {
    let query = sqlx::query_as!(Session, "select * from sessions")
        .fetch_all(connection)
        .await
        .map_err(|e| tide::Error::new(409, e))?;

    Ok(query)
}

#[derive(Deserialize, Serialize)]
pub struct AuthByMinute {
    pub time: Option<NaiveDateTime>,
    pub success: i64,
    pub failure: i64,
}

pub async fn recent_auth_attempts(
    connection: &mut MySqlConnection,
    hours: i64,
) -> tide::Result<Vec<AuthByMinute>> {
    let query = sqlx::query_as!(AuthByMinute, "SELECT STR_TO_DATE(DATE_FORMAT(starttime, '%Y-%m-%d %H:%i:00'), '%Y-%m-%d %H:%i:%s') AS time, COUNT(case auth.success when 1 then 1 else null end) AS success, COUNT(case auth.success when 0 then 1 else null end) AS failure from auth LEFT JOIN sessions ON auth.session=sessions.id WHERE sessions.starttime > DATE_ADD(NOW(), INTERVAL -? HOUR) GROUP BY time;", hours)
        .fetch_all(connection)
        .await
        .map_err(|e| tide::Error::new(409, e))?;

    Ok(query)
}
