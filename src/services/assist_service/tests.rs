use super::*;
use crate::{
    db,
    entities::user,
    models::memo_dto::CreateMemoRequest,
    services::memo_service::MemoService,
    test_utils::{MockGeminiClient, MockQdrantRepository},
};
use chrono::Utc;
use rand::Rng;
use sea_orm::*;

async fn setup_test_db() -> (Arc<DatabaseConnection>, i32) {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL_TEST")
        .expect("DATABASE_URL_TEST must be set. Run: just setup-test-db");
    let db = Arc::new(db::create_connection(&database_url).await.unwrap());

    let now = Utc::now().naive_utc();
    let timestamp = now.and_utc().timestamp_micros();
    let random: u32 = rand::thread_rng().gen();
    let unique_id = format!("{}_{}", timestamp, random);

    let new_user = user::ActiveModel {
        id: NotSet,
        username: Set(format!("test_user_{}", unique_id)),
        email: Set(format!("test_{}@example.com", unique_id)),
        password_hash: Set(Some("test_hash".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let user_id = new_user.insert(db.as_ref()).await.unwrap().id;

    (db, user_id)
}

#[tokio::test]
async fn test_get_assistance() {
    let (db, user_id) = setup_test_db().await;
    let qdrant_repo = Arc::new(MockQdrantRepository::new());
    let embedder = Arc::new(MockGeminiClient::new());
    let text_generator = Arc::new(MockGeminiClient::new());

    let memo_service = MemoService::new(
        db.clone(),
        qdrant_repo.clone(),
        embedder.clone() as Arc<dyn Embedder>,
    );

    memo_service
        .create_memo(
            user_id,
            CreateMemoRequest {
                content: "Rust is a systems programming language".to_string(),
            },
        )
        .await
        .unwrap();

    memo_service
        .create_memo(
            user_id,
            CreateMemoRequest {
                content: "Async programming in Rust".to_string(),
            },
        )
        .await
        .unwrap();

    let assist_service = AssistService::new(
        db.clone(),
        qdrant_repo as Arc<dyn QdrantRepo>,
        embedder as Arc<dyn Embedder>,
        text_generator as Arc<dyn TextGenerator>,
    );

    let req = AssistRequest {
        prompt: "Tell me about Rust programming".to_string(),
        limit: 5,
    };

    let result = assist_service.get_assistance(user_id, req).await.unwrap();

    assert!(!result.suggestion.is_empty());
    assert!(result.suggestion.contains("Tell me about Rust programming"));
}

#[tokio::test]
async fn test_get_assistance_no_similar_memos() {
    let (db, user_id) = setup_test_db().await;
    let qdrant_repo = Arc::new(MockQdrantRepository::new());
    let embedder = Arc::new(MockGeminiClient::new());
    let text_generator = Arc::new(MockGeminiClient::new());

    let assist_service = AssistService::new(
        db,
        qdrant_repo as Arc<dyn QdrantRepo>,
        embedder as Arc<dyn Embedder>,
        text_generator as Arc<dyn TextGenerator>,
    );

    let req = AssistRequest {
        prompt: "Tell me about Python".to_string(),
        limit: 5,
    };

    let result = assist_service.get_assistance(user_id, req).await.unwrap();

    assert!(!result.suggestion.is_empty());
    assert_eq!(result.similar_memos.len(), 0);
}

#[tokio::test]
async fn test_get_assistance_user_isolation() {
    let (db, user1_id) = setup_test_db().await;
    let (_, user2_id) = setup_test_db().await;

    let qdrant_repo = Arc::new(MockQdrantRepository::new());
    let embedder = Arc::new(MockGeminiClient::new());
    let text_generator = Arc::new(MockGeminiClient::new());

    let memo_service = MemoService::new(
        db.clone(),
        qdrant_repo.clone(),
        embedder.clone() as Arc<dyn Embedder>,
    );

    memo_service
        .create_memo(
            user1_id,
            CreateMemoRequest {
                content: "User 1 memo about Rust".to_string(),
            },
        )
        .await
        .unwrap();

    memo_service
        .create_memo(
            user2_id,
            CreateMemoRequest {
                content: "User 2 memo about Rust".to_string(),
            },
        )
        .await
        .unwrap();

    let assist_service = AssistService::new(
        db,
        qdrant_repo as Arc<dyn QdrantRepo>,
        embedder as Arc<dyn Embedder>,
        text_generator as Arc<dyn TextGenerator>,
    );

    let req = AssistRequest {
        prompt: "Tell me about Rust".to_string(),
        limit: 5,
    };

    let result = assist_service.get_assistance(user1_id, req).await.unwrap();

    assert!(result
        .similar_memos
        .iter()
        .all(|memo| memo.content.contains("User 1")));
    assert!(!result
        .similar_memos
        .iter()
        .any(|memo| memo.content.contains("User 2")));
}
