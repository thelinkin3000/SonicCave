use axum::{
    middleware,
    Router
    , routing::get
    ,
};
use tower_http::cors::{Any, CorsLayer};
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use id3::TagLike;
use md5::Digest;
use sea_orm::{ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel};
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};
use tokio::main;
use tokio_util::io::ReaderStream;
use entities::prelude::Song;

use migration::{Migrator, MigratorTrait};

use crate::auth_middleware::auth_middleware;
use crate::endpoint_handlers::get_artists;

mod auth_middleware;
mod database_sync;
mod explorer;
mod tag_parser;
mod endpoint_handlers;
mod responses;

async fn sync(connection: &DatabaseConnection) {
    println!("Gathering paths");
    let list = explorer::list("E:/Musica", 0).await;
    println!("Parsing tags");
    let hashmap = tag_parser::parse(list);
    println!("Syncing database");
    let ret = database_sync::sync_database(hashmap, connection).await;
    match (ret) {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
        }
    }
}

#[derive(Clone)]
pub struct DatabaseState {
    connection: DatabaseConnection,
}

#[main]
async fn main() -> Result<(), DbErr> {
    // Create a database connection
    let db = Database::connect("postgres://postgres:postgres@localhost/soniccave").await?;
    let state = DatabaseState {
        connection: db.to_owned(),
    };
    Migrator::up(&db, None).await?;
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(middleware::from_fn_with_state(state.to_owned(), auth_middleware))
        .route(
            "/sync",
            get(|| async {
                tokio::spawn(async move {
                    // `move` makes the closure take ownership of `slf`
                    sync(&db).await;
                });
                "Syncing. Check console output!"
            }),
        )
        .route("/song",get(get_song))
        .route("/rest/getArtists", get(get_artists))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(state.to_owned(), auth_middleware))
        .with_state(state.to_owned());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    Ok(axum::serve(listener, app).await.unwrap())
}

#[derive(Deserialize)]
struct IdQuery {
    id: i32,
}

#[axum::debug_handler]
async fn get_song(
    query: Option<Query<IdQuery>>,
    State(state): State<DatabaseState>,
) -> impl IntoResponse {
    if let None = query {
        return Err((StatusCode::NOT_FOUND, "No id of resource provided".to_string()));
    }
    let id = query.unwrap().id;
    let song_result = Song::find_by_id(id).one(&state.connection).await;
    if let Err(_) = song_result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error connecting to database".to_string()));
    }
    let song_option = song_result.unwrap();
    if let None = song_option {
        return Err((StatusCode::NOT_FOUND, "No song matching provided id".to_string()));
    }
    let song = song_option.unwrap();
    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(song.path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "text/toml; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment",
        ),
    ];

    Ok((headers, body))
}