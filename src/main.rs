use controllers::{sessions, views};
use dotenv::dotenv;
use sqlx::{mysql::MySql, Acquire};
use std::env;
use tera::Tera;
use tide_sqlx::{SQLxMiddleware, SQLxRequestExt};
use tide_tera::prelude::*;

mod controllers;
mod handlers;

#[derive(Clone, Debug)]
pub struct State {
    tera: Tera,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv().ok();
    tide::log::start();

    let mut tera = Tera::new("templates/**/*").expect("Error parsing templates directory");
    tera.autoescape_on(vec!["html"]);

    let state = State { tera };

    let mut app = tide::with_state(state);

    let database_url_string: String =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");
    app.with(SQLxMiddleware::<MySql>::new(&database_url_string).await?);

    app.at("/").get(views::index);
    app.at("/ip/:ip").get(views::ip_details);

    app.at("/api/auth/byminute")
        .get(controllers::sessions::auth_by_minute);

    app.at("/public")
        .serve_dir("./public/")
        .expect("Invalid static file directory");

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}
