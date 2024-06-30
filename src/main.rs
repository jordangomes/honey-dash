use controllers::views;
use sqlx::{mysql::MySql, Acquire};
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
    let mut tera = Tera::new("templates/**/*").expect("Error parsing templates directory");
    tera.autoescape_on(vec!["html"]);

    let state = State { tera };

    let mut app = tide::with_state(state);

    app.with(
        SQLxMiddleware::<MySql>::new("mysql://honeydash:F9fD7ThLFG9orddn@localhost:3306/cowrie")
            .await?,
    );

    app.at("/").get(views::index);

    app.at("/api/auth/byminute")
        .get(controllers::sessions::auth_by_minute);

    app.at("/public")
        .serve_dir("./public/")
        .expect("Invalid static file directory");

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}
