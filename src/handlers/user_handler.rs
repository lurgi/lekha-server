use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use super::AppState;
use crate::errors::ErrorResponse;
use crate::models::user_dto::{AuthResponse, OAuthLoginRequest};

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
    Json(payload): Json<OAuthLoginRequest>,
) -> impl IntoResponse {
    match state.user_service.oauth_login(payload).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => e.into_response(),
    }
}
