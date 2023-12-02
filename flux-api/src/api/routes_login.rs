use axum::{extract::Json, extract::State, response::Json as ResponseJson, routing::post, Router};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::cookie::time::OffsetDateTime;
use tower_cookies::{Cookie, Cookies};

use crate::api::helpers::password::hash_password;
use crate::controllers::sqlite::*;
use crate::error::{Error, Result};
use crate::jwt_claims::JwtClaims;
use crate::models::user::{FoundUser, PublicUser, User};
use crate::mw::AUTH_TOKEN;
use crate::server_config::ServerConfig;

pub fn routes(env_config: ServerConfig) -> Router {
    Router::new()
        .route("/login", post(handler_login))
        .route("/logout", post(handler_logout))
        .route("/sign-up", post(handler_sign_up))
        .with_state(env_config.clone())
}

async fn handler_login(
    cookies: Cookies,
    State(config): State<ServerConfig>,
    Json(login_user): Json<LoginPayload>,
) -> Result<Json<Value>> {
    // dbg!(&params);
    if let Some(payload_username) = login_user.username {
        let user = get_user_by_username(config.db_pool, payload_username).await?;
        if let Some(user) = user {
            if let Some(payload_password) = login_user.password {
                let found_password_and_salt = user.password.clone().unwrap();
                let (found_salt, found_password) =
                    found_password_and_salt.split_at(crate::api::helpers::SALT_LENGTH);
                let (hashed_password, _) = // we use the same salt as that we found in the db
                    hash_password(payload_password, Some(found_salt.to_string())); // do we need to offload this to a non-blocking thread?
                                                                                   // println!("hash={}\nsalt={}", hashed_password, salt);
                if hashed_password != found_password {
                    return Err(Error::AuthFailIncorrectPassword);
                }

                //password check passed
                let (jwt, _) = create_jwt(
                    user.username.clone().unwrap().as_str(),
                    config.jwt_secret_key,
                    user.permissions_group.unwrap(),
                );
                // Return the JWT token as a response
                let cookie = Cookie::build((AUTH_TOKEN, jwt)).path("/").build();
                cookies.add(cookie);
                let public_user = found_user_to_public_user(&user);
                let user_data = serde_json::to_value(public_user).unwrap();
                let body = ResponseJson(json!({
                    "result": {
                        "success": true,
                        "user_data": user_data,
                    }
                }));
                // sleep(Duration::from_secs(2)).await;
                Ok(body)
            } else {
                Err(Error::QueryFailNoPassword)
            }
        } else {
            Err(Error::AuthFailUserNotFound)
        }
    } else {
        Err(Error::QueryFailNoUsername)
    }
}

async fn handler_sign_up(
    cookies: Cookies,
    State(config): State<ServerConfig>,
    Json(new_user): Json<SignUpPayload>,
) -> Result<ResponseJson<Value>> {
    //TODO move validation to Validate crate (or garde)
    if let Some(payload_username) = new_user.username {
        let user = get_user_by_username(config.db_pool.clone(), payload_username.clone()).await?;
        // let user: Option<User> = None;
        if user.is_some() {
            Err(Error::SignUpFailUserAlreadyExists)
        } else if let Some(payload_password) = new_user.password {
            let (hashed_password, salt) = hash_password(payload_password.clone(), None); // do we need to offload this to a non-blocking thread?
                                                                                         // println!("hash={}\nsalt={}", hashed_password, salt);
            let salted_hashed_password = format!("{}{}", salt, hashed_password);
            println!(
                "{} - {}",
                new_user.name.clone().unwrap(),
                salted_hashed_password.clone()
            );

            let user = User {
                username: payload_username.clone(),
                password: salted_hashed_password,
                name: new_user.name.unwrap(),
                permissions_group: 0,
            };

            let created_user = create_new_user(config.db_pool, user).await;
            if created_user.is_ok() {
                let (jwt, _) = create_jwt(&payload_username, config.jwt_secret_key, 0);
                // Return the JWT token as a response
                let cookie = Cookie::build((AUTH_TOKEN, jwt)).path("/").build();
                cookies.add(cookie);
                let body = ResponseJson(json!({
                    "result": {
                        "success": true,
                    }
                }));
                Ok(body)
            } else {
                Err(created_user.err().unwrap())
            }
        } else {
            Err(Error::QueryFailNoPassword)
        }
    } else {
        Err(Error::QueryFailNoUsername)
    }
}

async fn handler_logout(cookies: Cookies) -> Result<ResponseJson<Value>> {
    // Return the JWT token as a response
    let cookie = Cookie::build((AUTH_TOKEN, ""))
        .path("/")
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build();
    cookies.add(cookie);
    let body = ResponseJson(json!({
        "result": {
            "success": true,
        }
    }));

    Ok(body)
}

pub fn create_jwt(username: &str, jwt_secret_key: String, group: i64) -> (String, u64) {
    let claims = &JwtClaims::new(username, group);
    (
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(jwt_secret_key.as_ref()),
        )
        .unwrap(),
        claims.exp,
    )
}

fn found_user_to_public_user(found_user: &FoundUser) -> PublicUser {
    PublicUser {
        id: found_user.id.unwrap(),
        username: found_user.username.as_ref().unwrap().to_string(),
        name: found_user.name.as_ref().unwrap().to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SignUpPayload {
    name: Option<String>,
    username: Option<String>,
    password: Option<String>,
}
