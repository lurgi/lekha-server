use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::entities::oauth_account::OAuthProvider;
use crate::errors::ErrorResponse;
use crate::handlers::health_handler::HealthResponse;
use crate::models::assist_dto::{AssistRequest, AssistResponse, SimilarMemo};
use crate::models::memo_dto::{CreateMemoRequest, MemoResponse, UpdateMemoRequest};
use crate::models::user_dto::{AuthResponse, OAuthLoginRequest, UserResponse};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Lekha Server API",
        version = "0.1.0",
        description = "개인 메모 관리 시스템 with AI\n\n## 인증\n현재 임시 인증 방식으로 `X-User-Id` 헤더를 사용합니다.\n향후 OAuth 토큰 기반 인증으로 교체될 예정입니다."
    ),
    paths(
        crate::handlers::health_handler::health_check,
        crate::handlers::user_handler::oauth_login,
        crate::handlers::memo_handler::create_memo,
        crate::handlers::memo_handler::list_memos,
        crate::handlers::memo_handler::get_memo,
        crate::handlers::memo_handler::update_memo,
        crate::handlers::memo_handler::delete_memo,
        crate::handlers::memo_handler::toggle_pin,
        crate::handlers::assist_handler::assist,
    ),
    components(
        schemas(
            HealthResponse,
            OAuthLoginRequest,
            UserResponse,
            AuthResponse,
            OAuthProvider,
            CreateMemoRequest,
            UpdateMemoRequest,
            MemoResponse,
            AssistRequest,
            AssistResponse,
            SimilarMemo,
            ErrorResponse,
        )
    ),
    tags(
        (name = "Health", description = "서버 상태 확인"),
        (name = "Users", description = "사용자 관리"),
        (name = "Memos", description = "메모 관리"),
        (name = "Assist", description = "AI 어시스턴트"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}
