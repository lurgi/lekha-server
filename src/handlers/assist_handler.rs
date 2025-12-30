use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use super::{auth::AuthenticatedUser, AppState};
use crate::models::assist_dto::AssistRequest;

pub async fn assist(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AssistRequest>,
) -> impl IntoResponse {
    match state.assist_service.get_assistance(user.id, payload).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => e.into_response(),
    }
}
