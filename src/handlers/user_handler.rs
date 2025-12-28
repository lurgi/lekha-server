use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use super::AppState;
use crate::models::user_dto::OAuthLoginRequest;

pub async fn oauth_login(
    State(state): State<AppState>,
    Json(payload): Json<OAuthLoginRequest>,
) -> impl IntoResponse {
    match state.user_service.oauth_login(payload).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => e.into_response(),
    }
}
