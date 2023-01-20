use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, http::Response, Form};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use rand::rngs::OsRng;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use nori::entities::{session, user};

use crate::AppState;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct UserInfoArgs {
    pub username: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    user_data: Form<UserInfoArgs>,
) -> (PrivateCookieJar, Response<String>) {
    let conn = state.db;
    let existing = user::Entity::find()
        .filter(user::Column::Username.eq(user_data.username.clone()))
        .all(&conn)
        .await;
    match existing {
        Ok(users) if !users.is_empty() => (
            jar,
            Response::builder()
                .status(400)
                .header("Content-Type", "text/html")
                .body("Username already exists".to_owned())
                .unwrap(),
        ),
        Err(e) => (
            jar,
            Response::builder()
                .status(500)
                .header("Content-Type", "text/html")
                .body(format!("Internal Server Error: {}", e))
                .unwrap(),
        ),
        _ => {
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
                let res = Response::builder()
                    .status(500)
                    .header("Content-Type", "text/html")
                    .body("Internal Server Error".to_owned())
                    .unwrap();
                return (jar, res);
            }
            let inserted_user = insert_res.unwrap();

            let session = session::ActiveModel::create_session(inserted_user.last_insert_id, None);

            let insert_session_res = session::Entity::insert(session).exec(&conn).await;
            if insert_session_res.is_err() {
                println!("Error inserting session: {:?}", insert_session_res);
                let res = Response::builder()
                    .status(500)
                    .header("Content-Type", "text/html")
                    .body("Internal Server Error".to_owned())
                    .unwrap();
                return (jar, res);
            }
            let inserted_session = insert_session_res.unwrap();
            let jar = jar.add(
                Cookie::build("session_id", inserted_session.last_insert_id.to_string())
                    .http_only(true)
                    .secure(true)
                    .finish(),
            );

            (
                jar,
                Response::builder()
                    .status(200)
                    .header("Content-Type", "text/html")
                    .body("User created".to_owned())
                    .unwrap(),
            )
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    user_data: Form<UserInfoArgs>,
) -> (PrivateCookieJar, Response<String>) {
    let conn = state.db;
    if jar.get("session_id").is_some() {
        return (
            jar,
            Response::builder()
                .status(400)
                .header("Content-Type", "text/html")
                .body("Already logged in".to_owned())
                .unwrap(),
        );
    }
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
                let session = session::ActiveModel::create_session(user.id, None);

                let insert_session_res = session::Entity::insert(session).exec(&conn).await;
                if insert_session_res.is_err() {
                    println!("Error inserting session: {:?}", insert_session_res);
                    let res = Response::builder()
                        .status(500)
                        .header("Content-Type", "text/html")
                        .body("Internal Server Error".to_owned())
                        .unwrap();
                    return (jar, res);
                }
                let inserted_session = insert_session_res.unwrap();
                let jar = jar.add(
                    Cookie::build("session_id", inserted_session.last_insert_id.to_string())
                        .http_only(true)
                        .secure(true)
                        .finish(),
                );
                
                (
                    jar,
                    Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .body("Login successful".to_owned())
                        .unwrap(),
                )
            } else {
                (
                    jar,
                    Response::builder()
                        .status(400)
                        .header("Content-Type", "text/html")
                        .body("Incorrect login".to_owned())
                        .unwrap(),
                )
            }
        }
        Err(e) => (
            jar,
            Response::builder()
                .status(500)
                .header("Content-Type", "text/html")
                .body(format!("Internal Server Error: {}", e))
                .unwrap(),
        ),
        _ => (
            jar,
            Response::builder()
                .status(400)
                .header("Content-Type", "text/html")
                .body("User does not exist".to_owned())
                .unwrap(),
        ),
    }
}
