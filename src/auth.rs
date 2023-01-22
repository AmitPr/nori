use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::Json;

use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use nori::json::{JsonBuilder, JsonResult};
use rand::rngs::OsRng;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use nori::entities::{session, user};
use thiserror::Error;
use tracing::warn;

use crate::AppState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Error)]
pub enum AuthError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Incorrect username or password")]
    IncorrectCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Session expired")]
    SessionExpired,
    #[error("Internal server error")]
    InternalServerError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub remember: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub session_id: String,
}

pub async fn register(
    jar: CookieJar,
    State(state): State<AppState>,
    user_data: Json<AuthRequest>,
) -> JsonResult<AuthResponse, AuthError> {
    let conn = state.db;
    let existing = user::Entity::find()
        .filter(user::Column::Username.eq(user_data.username.clone()))
        .all(&conn)
        .await
        .map(|data| data.is_empty());
    tracing::info!("jar: {:?}", jar);
    match existing {
        Ok(false) => JsonBuilder::new(Err(AuthError::UserAlreadyExists))
            .status(StatusCode::CONFLICT)
            .build(),
        Err(_) => Err(AuthError::InternalServerError).into(),
        Ok(true) => {
            //TODO: Validate username and password match requirements
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(user_data.password.as_bytes(), &salt)
                .expect("could not hash password")
                .to_string();

            let u = user::ActiveModel {
                id: NotSet,
                username: Set(user_data.username.clone()),
                password_hash: Set(password_hash),
            };

            let insert_res = user::Entity::insert(u).exec(&conn).await;
            if insert_res.is_err() {
                warn!("Error inserting user: {:?}", insert_res);
                return Err(AuthError::InternalServerError).into();
            }
            let inserted_user = insert_res.unwrap();

            let expiry_time = if user_data.remember.unwrap_or(false) {
                None
            } else {
                chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::seconds(10)) //TODO
                    .map(|t| t.into())
            };

            let session =
                session::ActiveModel::create_session(inserted_user.last_insert_id, expiry_time);
            let header = session.get_cookie();

            let insert_session_res = session::Entity::insert(session).exec(&conn).await;
            if insert_session_res.is_err() {
                warn!("Error creating session: {:?}", insert_session_res);
                return Err(AuthError::InternalServerError).into();
            }
            let inserted_session = insert_session_res.unwrap();

            JsonBuilder::new(Ok(AuthResponse {
                session_id: inserted_session.last_insert_id.to_string(),
            }))
            .status(StatusCode::CREATED)
            .header("Set-Cookie", header)
            .build()
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    user_data: Json<AuthRequest>,
) -> JsonResult<AuthResponse, AuthError> {
    let conn = state.db;
    let existing = user::Entity::find()
        .filter(user::Column::Username.eq(user_data.username.clone()))
        .one(&conn)
        .await;
    match existing {
        Ok(Some(user)) => {
            let parsed_hash = PasswordHash::new(&user.password_hash)
                .expect("could not parse stored password hash");
            if Argon2::default()
                .verify_password(user_data.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                let expiry_time = if user_data.remember.unwrap_or(false) {
                    None
                } else {
                    chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::seconds(10)) //TODO
                        .map(|t| t.into())
                };
                let session = session::ActiveModel::create_session(user.id, expiry_time);
                let header = session.get_cookie();

                let insert_session_res = session::Entity::insert(session).exec(&conn).await;
                if insert_session_res.is_err() {
                    warn!("Error creating session: {:?}", insert_session_res);
                    return Err(AuthError::InternalServerError).into();
                }
                let inserted_session = insert_session_res.unwrap();

                JsonBuilder::new(Ok(AuthResponse {
                    session_id: inserted_session.last_insert_id.to_string(),
                }))
                .header("Set-Cookie", header)
                .build()
            } else {
                JsonBuilder::new(Err(AuthError::IncorrectCredentials))
                    .status(StatusCode::UNAUTHORIZED)
                    .build()
            }
        }
        Err(e) => {
            warn!("Error finding user: {:?}", e);
            Err(AuthError::InternalServerError).into()
        }
        _ => JsonBuilder::new(Err(AuthError::InvalidCredentials))
            .status(StatusCode::UNAUTHORIZED)
            .build(),
    }
}
