use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use md5::{Digest, Md5};
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use serde::Deserialize;

use entities::prelude::User;
use entities::user;

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
        return Auth {
            u: "".to_string(),
            t: "".to_string(),
            s: "".to_string(),
            v: "".to_string(),
            c: "".to_string(),
            f: "".to_string(),
        };
    }
}

pub async fn auth_middleware(
    State(state): State<DatabaseState>,
    auth: Option<Query<Auth>>,
    request: Request,
    next: Next,
) -> Response {
    // do something with `request`...
    println!("hey im here");
    let owned_auth = auth.unwrap_or_default().to_owned();
    let user_result = User::find().filter(user::Column::Username.eq(&owned_auth.u)).one(&state.connection).await;
    if let Err(_) = user_result {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user_option = user_result.unwrap();
    if let None = user_option {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user = user_option.unwrap();

    println!("user exists!");
    // create a Md5 hasher instance
    let mut hasher = Md5::new();

    // process input message
    hasher.update(user.password + &*owned_auth.s);

    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 16]
    let result = hasher.finalize();
    if !owned_auth.t.eq(&format!("{:x}", result)) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Carry on my wayward son
    let response = next.run(request).await;

    response
}