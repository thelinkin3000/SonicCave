use std::env::args;
use std::str::FromStr;
use std::{env, fs};

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{middleware, routing::get, Router};
use clap::Parser;
use id3::TagLike;
use log::{error, info};
use md5::Digest;
use sea_orm::prelude::Uuid;
use sea_orm::QueryFilter;
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use stopwatch::Stopwatch;
use tokio::main;
use tokio_util::io::ReaderStream;
use tower_http::cors::{Any, CorsLayer};

use entities::prelude::Song;
use entities::song;
use migration::{Migrator, MigratorTrait};

use crate::auth_middleware::auth_middleware;
use crate::endpoint_handlers::{get_album, get_albums, get_artist, get_artists, search};

mod auth_middleware;
mod database_sync;
mod endpoint_handlers;
mod explorer;
mod responses;
mod tag_parser;

async fn sync(connection: &DatabaseConnection) {
    info!("Gathering paths");
    let list = explorer::list("E:/Musica", 0).await;
    info!("Parsing tags");
    let hashmap = tag_parser::parse(list);
    info!("Syncing database");
    let ret = database_sync::sync_database(hashmap, connection).await;
    match ret {
        Ok(_) => {}
        Err(error) => {
            error!("{}", error);
        }
    }
}

#[derive(Clone)]
pub struct DatabaseState {
    connection: DatabaseConnection,
    pool: Pool<Postgres>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, short, default_value_t = 3)]
    verbosity: usize,
    #[arg(long, short, default_value_t = false)]
    quiet: bool,
    #[arg(long, short)]
    config: String,
}

#[derive(Deserialize)]
struct Config {
    port: i32,
    path: String,
    postgres: String,
}

#[main]
async fn main() -> Result<(), DbErr> {
    let args = Args::parse();
    stderrlog::new()
        .verbosity(args.verbosity)
        .quiet(args.quiet)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    info!("Configuration path: {}", args.config);
    let config_string_result = fs::read_to_string(args.config);
    if let Err(err) = config_string_result {
        error!("Error opening configuration file: {}", err);
        return Ok(());
    }
    let config_string = config_string_result.unwrap();
    let config_result = serde_json::from_str(config_string.as_str());
    if let Err(err) = config_result {
        error!("Malformed configuration: {}", err);
        return Ok(());
    }
    let config: Config = config_result.unwrap();
    let mut connection_options = ConnectOptions::new(config.postgres.to_owned());
    connection_options.sqlx_logging_level(log::LevelFilter::Trace); // Or set SQLx log level
                                                                    // Create a database connection
    let db_result = Database::connect(connection_options).await;
    if let Err(err) = db_result {
        error!("Error connecting to database: {}", err);
        return Ok(());
    }
    let db = db_result.unwrap();
    let pool_result = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.postgres.to_owned().as_str())
        .await;
    if let Err(err) = pool_result {
        error!("Error connecting to database: {}", err);
        return Ok(());
    }
    let pool = pool_result.unwrap();
    let state = DatabaseState {
        connection: db.to_owned(),
        pool: pool.to_owned(),
    };
    Migrator::up(&db, None).await?;
    // build our application with a single route

    let authenticated: Router = Router::new()
        // Root
        .route("/", get(|| async { "Hello, World!" }))
        // Stream
        .route("/stream", get(get_stream))
        .route("/getArtists", get(get_artists))
        .route("/getArtist", get(get_artist))
        .route("/search3", get(search))
        .route("/getAlbumList2", get(get_albums))
        .route("/getAlbum", get(get_album))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(
            state.to_owned(),
            auth_middleware,
        ))
        .with_state(state.to_owned());
    let app: Router = Router::new()
        .route("/search", get(search))
        // StartScan
        .route(
            "/startScan",
            get(|| async {
                tokio::spawn(async move {
                    // `move` makes the closure take ownership of `slf`
                    sync(&db).await;
                });
                "Syncing. Check console output!"
            }),
        )
        .with_state(state.to_owned())
        .nest("/rest", authenticated);

    // Welcome messages
    info!(
        r#"
                                      -
                                     +*+
                                   .+****:
                                  :*******-
                                 -*********=               .:-
                                =***********+.      .:-=+*****.
                               =*************+.-=+************.
                             .+**************  -**********+++*.
                       -=:  .****************   :**++=-:.   =*.
                      =****=*****************  -++. :++     =*.
                     =***********************  =***+***+    =*.
                   .+************************  =*******+-.: =*.
                  .**************************  =*****-    +***.
                 :***********************+++*  =****-      =**.
                -*********************=.       =****:       -*.
               =*********************=         =*****:       =
              +**********************-         =******+-::-=**+  .-
         =*-.*************************.       :****************+=**=
       .+******************************+-:::-+**********************+.
      :***************************************************************.
     :*****************************************************************:
    =*******************************************************************-
   +*********************************************************************=
 .+***********************************************************************+."#
    );
    info!("Listening on 0.0.0.0: {}", config.port);
    info!("Welcome to SonicCave!");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    Ok(axum::serve(listener, app).await.unwrap())
}

#[derive(Deserialize)]
struct IdQuery {
    id: Uuid,
}

#[axum::debug_handler]
async fn get_stream(
    query: Option<Query<IdQuery>>,
    State(state): State<DatabaseState>,
) -> impl IntoResponse {
    if let None = query {
        return Err((
            StatusCode::NOT_FOUND,
            "No id of resource provided".to_string(),
        ));
    }
    let id = query.unwrap().id;

    let song_result = Song::find()
        .filter(song::Column::Id.eq(id))
        .one(&state.connection)
        .await;

    if let Err(_) = song_result {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error connecting to database".to_string(),
        ));
    }
    let song_option = song_result.unwrap();
    if let None = song_option {
        return Err((
            StatusCode::NOT_FOUND,
            "No song matching provided id".to_string(),
        ));
    }
    let song = song_option.unwrap();
    info!("Streaming song {} with id {}", song.title, song.id);

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
        (header::X_CONTENT_TYPE_OPTIONS, "no-sniff"),
        (
            header::HeaderName::from_str("X-Content-Duration").unwrap(),
            "0.0",
        ),
    ];

    Ok((headers, body))
}
