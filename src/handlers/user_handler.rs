use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tower_cookies::{Cookie, Cookies};

use super::AppState;
use crate::errors::ErrorResponse;
use crate::models::user_dto::{AuthResponse, OAuthLoginRequest};
use crate::services::TokenService;

#[utoipa::path(
    post,
    path = "/api/users/oauth-login",
    tag = "Users",
    request_body = OAuthLoginRequest,
    responses(
        (status = 200, description = "로그인 성공", body = AuthResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    )
)]
pub async fn oauth_login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<OAuthLoginRequest>,
) -> impl IntoResponse {
    match state.user_service.oauth_login(payload).await {
        Ok((auth_response, access_token, refresh_token)) => {
            let is_production = std::env::var("ENV")
                .unwrap_or_else(|_| "development".to_string())
                == "production";

            let same_site = if is_production {
                tower_cookies::cookie::SameSite::None
            } else {
                tower_cookies::cookie::SameSite::Lax
            };

            let access_cookie = Cookie::build(("access_token", access_token))
                .http_only(true)
                .secure(is_production)
                .same_site(same_site)
                .max_age(time::Duration::seconds(TokenService::access_token_max_age()))
                .path("/")
                .build();

            let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .secure(is_production)
                .same_site(same_site)
                .max_age(time::Duration::seconds(
                    TokenService::refresh_token_max_age(),
                ))
                .path("/")
                .build();

            cookies.add(access_cookie);
            cookies.add(refresh_cookie);

            (StatusCode::OK, Json(auth_response)).into_response()
        }
        Err(e) => e.into_response(),
    }
}
