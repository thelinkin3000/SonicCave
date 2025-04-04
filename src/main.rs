use std::fs;
use std::str::FromStr;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{middleware, routing::get, Router};
use clap::Parser;
use log::{error, info};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

use tokio::main;
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::auth_middleware::auth_middleware;
use crate::endpoint_handlers::{
    create_update_playlist, get_album, get_albums, get_artist, get_artists, get_playlist,
    get_playlists, search,
};

mod auth_middleware;
mod database_sync;
mod endpoint_handlers;
mod explorer;
mod responses;
mod tag_parser;

async fn sync(connection: &mut Pool<Postgres>, path: &str) {
    info!("Gathering paths");
    let list = explorer::list(path, 0, true).await;
    info!("Parsing tags");
    let hashmap_result = tag_parser::parse(list);
    let hashmap;
    match hashmap_result {
        Ok(h) => hashmap = h,
        Err(_) => {
            error!("Failed to parse tags");
            return;
        }
    }
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
async fn main() -> Result<(), sqlx::Error> {
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
    let pool_result = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.postgres.to_owned().as_str())
        .await;
    if let Err(err) = pool_result {
        error!("Error connecting to database: {}", err);
        return Ok(());
    }
    let mut pool = pool_result.unwrap();
    let state = DatabaseState {
        pool: pool.to_owned(),
    };
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
        .route("/getPlaylists", get(get_playlists))
        .route("/getPlaylist", get(get_playlist))
        .route("/createPlaylist", get(create_update_playlist))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(
            state.to_owned(),
            auth_middleware,
        ))
        .with_state(state.to_owned());
    let app: Router = Router::new()
        .route("/search", get(search))
        .route("/playlist", get(create_update_playlist))
        .route("/playlists", get(get_playlists))
        // StartScan
        .route(
            "/startScan",
            get(|| async {
                tokio::spawn(async move {
                    // `move` makes the closure take ownership of `slf`
                    sync(&mut pool, &config.path.as_str()).await;
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

    let song_result = queries::get_song_by_id(&state.pool, id).await;

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
