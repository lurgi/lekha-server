use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tower_cookies::{Cookie, Cookies};

use super::{auth::AuthenticatedUser, AppState};
use crate::{
    errors::ErrorResponse,
    models::user_dto::{AuthResponse, LogoutResponse},
    services::TokenService,
};

fn get_cookie_config() -> (bool, tower_cookies::cookie::SameSite) {
    let is_production = std::env::var("ENV")
        .unwrap_or_else(|_| "development".to_string())
        == "production";

    let same_site = if is_production {
        tower_cookies::cookie::SameSite::None
    } else {
        tower_cookies::cookie::SameSite::Lax
    };

    (is_production, same_site)
}

fn build_access_token_cookie(token: &str, is_production: bool, same_site: tower_cookies::cookie::SameSite) -> Cookie<'static> {
    Cookie::build(("access_token", token.to_string()))
        .http_only(true)
        .secure(is_production)
        .same_site(same_site)
        .max_age(time::Duration::seconds(TokenService::access_token_max_age()))
        .path("/")
        .build()
}

fn build_refresh_token_cookie(token: &str, is_production: bool, same_site: tower_cookies::cookie::SameSite) -> Cookie<'static> {
    Cookie::build(("refresh_token", token.to_string()))
        .http_only(true)
        .secure(is_production)
        .same_site(same_site)
        .max_age(time::Duration::seconds(TokenService::refresh_token_max_age()))
        .path("/")
        .build()
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    responses(
        (status = 200, description = "토큰 재발급 성공", body = AuthResponse),
        (status = 401, description = "Refresh Token 만료 또는 유효하지 않음", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let refresh_token_cookie = match cookies.get("refresh_token") {
        Some(cookie) => cookie,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing refresh token" })),
            )
                .into_response()
        }
    };

    let refresh_token = refresh_token_cookie.value();

    match state.user_service.refresh_tokens(refresh_token).await {
        Ok((access_token, new_refresh_token, _user_id)) => {
            let (is_production, same_site) = get_cookie_config();

            let access_cookie = build_access_token_cookie(&access_token, is_production, same_site);
            let refresh_cookie = build_refresh_token_cookie(&new_refresh_token, is_production, same_site);

            cookies.add(access_cookie);
            cookies.add(refresh_cookie);

            (
                StatusCode::OK,
                Json(serde_json::json!({ "message": "Token refreshed successfully" })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "로그아웃 성공", body = LogoutResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let refresh_token_cookie = match cookies.get("refresh_token") {
        Some(cookie) => cookie,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing refresh token" })),
            )
                .into_response()
        }
    };

    let refresh_token = refresh_token_cookie.value();

    match state.user_service.logout(refresh_token).await {
        Ok(_) => {
            cookies.remove(Cookie::from("access_token"));
            cookies.remove(Cookie::from("refresh_token"));

            (
                StatusCode::OK,
                Json(LogoutResponse {
                    message: "Successfully logged out".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/auth/logout-all",
    tag = "Auth",
    responses(
        (status = 200, description = "모든 디바이스 로그아웃 성공", body = LogoutResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    )
)]
pub async fn logout_all(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    cookies: Cookies,
) -> impl IntoResponse {
    match state.user_service.logout_all(user.id).await {
        Ok(_) => {
            cookies.remove(Cookie::from("access_token"));
            cookies.remove(Cookie::from("refresh_token"));

            (
                StatusCode::OK,
                Json(LogoutResponse {
                    message: "Successfully logged out from all devices".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}
