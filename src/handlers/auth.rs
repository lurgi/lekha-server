use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::utils::jwt;

#[derive(Debug, Deserialize)]
pub struct AuthenticatedUser {
    pub id: i32,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse cookies"))?;

        let cookie = cookies
            .get("access_token")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing access token"))?;

        let token = cookie.value();

        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "JWT secret not configured",
            )
        })?;

        let claims = jwt::verify_token(token, &jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

        let id = claims
            .sub
            .parse::<i32>()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID in token"))?;

        Ok(AuthenticatedUser { id })
    }
}
