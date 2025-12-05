use std::{env, future::{Ready, ready}};

use actix_web::{
    dev::Payload, error::ErrorUnauthorized,  FromRequest, HttpRequest,
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::models::Claims;

pub struct JwtClaims (pub Claims);

impl FromRequest for JwtClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self,Self::Error>>;

        fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(header_value) = auth_header {
            if let Ok(token) = header_value.to_str() {
                let secret = env::var("SECRET_KEY").expect("JWT_SECRET must be set");
                let decoding_key = DecodingKey::from_secret(secret.as_bytes());
                let validation = Validation::default();

                match decode::<Claims>(token, &decoding_key, &validation) {
                    Ok(token_data) => {
                        return ready(Ok(JwtClaims(token_data.claims)));
                    }
                    Err(e) => {
                        eprintln!("JWT decoding error: {:?}", e);
                        return ready(Err(ErrorUnauthorized("Invalid JWT token")));
                    }
                }
            }
        }
        ready(Err(ErrorUnauthorized("Authorization header missing or invalid")))
    }
}