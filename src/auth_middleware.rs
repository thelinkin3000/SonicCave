use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use log::{error, warn};
use md5::{Digest, Md5};
use serde::Deserialize;

use crate::DatabaseState;

#[derive(Deserialize, Clone)]
pub struct Auth {
    u: String,
    t: String,
    s: String,
    v: String,
    c: String,
    f: String,
}

impl Default for Auth {
    fn default() -> Self {
        Auth {
            u: "".to_string(),
            t: "".to_string(),
            s: "".to_string(),
            v: "".to_string(),
            c: "".to_string(),
            f: "".to_string(),
        }
    }
}

pub async fn auth_middleware(
    State(state): State<DatabaseState>,
    auth: Option<Query<Auth>>,
    request: Request,
    next: Next,
) -> Response {
    // do something with `request`...
    let owned_auth = auth.unwrap_or_default().to_owned();
    let user_result = queries::get_user_by_username(&state.pool, &owned_auth.u).await;
    if let Err(err) = user_result {
        error!("Error in database connection: {}", err);
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user_option = user_result.unwrap();
    if user_option.is_none() {
        warn!("User doesn't exist: {}", &owned_auth.u);
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user = user_option.unwrap();

    // create a Md5 hasher instance
    let mut hasher = Md5::new();

    // process input message
    hasher.update(user.password + &*owned_auth.s);

    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 16]
    let result = hasher.finalize();
    if !owned_auth.t.eq(&format!("{:x}", result)) {
        warn!("Wrong password for user {}", &owned_auth.u);
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Carry on my wayward son

    next.run(request).await
}
