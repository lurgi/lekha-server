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
        description = "당신의 생각이 글이 되도록 돕습니다\n\n## 인증\nOAuth 소셜 로그인(Google, Kakao, Naver)을 통해 사용자 인증을 수행합니다.\n로그인 후 발급받은 JWT Access Token을 `Authorization: Bearer <token>` 헤더에 포함하여 API를 호출합니다."
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
