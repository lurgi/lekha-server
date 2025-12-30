use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use inklings_server::{
    clients::{Embedder, TextGenerator},
    db,
    entities::user,
    handlers,
    models::memo_dto::{CreateMemoRequest, MemoResponse},
    services,
    test_utils::{MockGeminiClient, MockQdrantRepository},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;
use tower::util::ServiceExt;

async fn setup() -> (Router, Arc<DatabaseConnection>) {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL_TEST")
        .expect("DATABASE_URL_TEST must be set. Run: just setup-test-db");

    let db = Arc::new(db::create_connection(&database_url).await.unwrap());

    let qdrant_repo = Arc::new(MockQdrantRepository::new());
    let gemini_client = Arc::new(MockGeminiClient::new());

    let app = handlers::create_router(
        db.clone(),
        qdrant_repo,
        gemini_client.clone() as Arc<dyn Embedder>,
        gemini_client as Arc<dyn TextGenerator>,
    );
    (app, db)
}

async fn create_test_user(db: &DatabaseConnection, id: i32, username: &str) -> user::Model {
    let _ = user::Entity::delete_by_id(id).exec(db).await;

    let user = user::ActiveModel {
        id: Set(id),
        username: Set(username.to_owned()),
        email: Set(format!("{}@test.com", username)),
        password_hash: Set(Some("hashed_password".to_owned())),
        ..Default::default()
    };
    user.insert(db).await.unwrap()
}

#[tokio::test]
async fn test_create_memo_api() {
    let (app, db) = setup().await;
    let user = create_test_user(&db, 1, "user1").await;

    let req_body = CreateMemoRequest {
        content: "Test memo from integration test".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/memos")
                .header(http::header::CONTENT_TYPE, "application/json")
                .header("X-User-Id", user.id.to_string())
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let memo_res: MemoResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(memo_res.content, req_body.content);
    assert_eq!(memo_res.user_id, user.id);
}

#[tokio::test]
async fn test_list_memos_api() {
    let (app, db) = setup().await;
    let user1 = create_test_user(&db, 10, "user10").await;
    let user2 = create_test_user(&db, 20, "user20").await;

    let qdrant_repo = Arc::new(MockQdrantRepository::new());
    let embedder = Arc::new(MockGeminiClient::new());
    let memo_service = Arc::new(services::memo_service::MemoService::new(
        db.clone(),
        qdrant_repo,
        embedder as Arc<dyn Embedder>,
    ));

    memo_service
        .create_memo(
            user1.id,
            CreateMemoRequest {
                content: "user1 memo 1".to_string(),
            },
        )
        .await
        .unwrap();
    memo_service
        .create_memo(
            user1.id,
            CreateMemoRequest {
                content: "user1 memo 2".to_string(),
            },
        )
        .await
        .unwrap();
    memo_service
        .create_memo(
            user2.id,
            CreateMemoRequest {
                content: "user2 memo".to_string(),
            },
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/memos")
                .header("X-User-Id", user1.id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let memos: Vec<MemoResponse> = serde_json::from_slice(&body).unwrap();

    assert_eq!(memos.len(), 2);
    assert!(memos.iter().all(|m| m.user_id == user1.id));
}

#[tokio::test]
async fn test_get_memo_unauthorized_api() {
    let (app, db) = setup().await;
    let user1 = create_test_user(&db, 30, "user30").await;
    let user2 = create_test_user(&db, 40, "user40").await;

    let qdrant_repo = Arc::new(MockQdrantRepository::new());
    let embedder = Arc::new(MockGeminiClient::new());
    let memo_service = Arc::new(services::memo_service::MemoService::new(
        db.clone(),
        qdrant_repo,
        embedder as Arc<dyn Embedder>,
    ));

    let memo1 = memo_service
        .create_memo(
            user1.id,
            CreateMemoRequest {
                content: "user1's secret memo".to_string(),
            },
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/api/memos/{}", memo1.id))
                .header("X-User-Id", user2.id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
