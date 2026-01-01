use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use super::{auth::AuthenticatedUser, AppState};
use crate::models::assist_dto::{AssistRequest, AssistResponse};
use crate::errors::ErrorResponse;

#[utoipa::path(
    post,
    path = "/api/assist",
    tag = "Assist",
    request_body = AssistRequest,
    responses(
        (status = 200, description = "AI 어시스턴트 응답 성공", body = AssistResponse),
        (status = 400, description = "잘못된 요청", body = ErrorResponse),
        (status = 401, description = "인증 실패", body = ErrorResponse),
        (status = 500, description = "서버 에러", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
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
