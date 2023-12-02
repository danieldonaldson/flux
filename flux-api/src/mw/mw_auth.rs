use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::routes_login::create_jwt;
use crate::jwt_claims::{JwtClaims, REFRESH_TIME_BEFORE_EXPIRY};
use crate::server_config::ServerConfig;
use crate::{error::Error, error::Result, mw::AUTH_TOKEN};

use axum::extract::State;
use axum::{extract::Request, http, middleware::Next, response::Response};
use axum_macros::debug_handler;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tower_cookies::{Cookie, Cookies};

pub async fn mw_require_auth(
    State(config): State<ServerConfig>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
    // dbg!(&auth_token);
    let auth_token = auth_token.ok_or(Error::AuthFailNoAuthTokenCookie)?;

    let (username, mut exp, permissions_group) = parse_token(auth_token, &config.jwt_secret_key)?;

    // check if we need a refresh
    let current_time = get_current_time();
    if exp - REFRESH_TIME_BEFORE_EXPIRY < current_time {
        // generate new jwt
        // Add check against database to see if this username has been revoked?
        let (jwt, new_exp) = create_jwt(&username, config.jwt_secret_key, permissions_group);
        cookies.add(Cookie::new(AUTH_TOKEN, jwt));
        exp = new_exp;
        // set it in the cookie
        // println!("Token refreshed for {}", username);
    }

    let jwt = JwtClaims {
        username,
        exp,
        permissions_group,
    };
    req.extensions_mut().insert(jwt);

    Ok(next.run(req).await)
}

// pub async fn mw_require_user_or_higher<B>(req: Request<B>, next: Next<B>) -> Result<Response> {
//     let jwt = req.extensions().get::<JwtClaims>().unwrap();
//     dbg!("--> {} ", jwt);
//     if jwt.permissions_group < 1 {
//         eprint!("Group too low - user");
//         return Err(Error::AuthFailGroupTooLow);
//     }

//     Ok(next.run(req).await)
// }

// pub async fn mw_require_parent_or_higher<B>(req: Request<B>, next: Next<B>) -> Result<Response> {
//     let jwt = req.extensions().get::<JwtClaims>().unwrap();
//     dbg!("--> {} ", jwt);
//     if jwt.permissions_group < 2 {
//         eprint!("Group too low - parent");
//         return Err(Error::AuthFailGroupTooLow);
//     }

//     Ok(next.run(req).await)
// }

fn parse_token(token: String, secret: &String) -> Result<(String, u64, i64)> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let token_data = decode::<JwtClaims>(&token, &decoding_key, &Validation::default());

    if let Ok(token) = token_data {
        let expiration_time = token.claims.exp;

        let current_time = get_current_time();

        if expiration_time < current_time {
            return Err(Error::AuthFailTokenExpired);
        }

        // Retrieve the username from the JWT claims
        let username = token.claims.username;

        // Use the username as needed
        // println!("Username: {}", username);
        Ok((username, expiration_time, token.claims.permissions_group))
    } else {
        Err(Error::AuthFailTokenWrongFormat)
    }
}

fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
