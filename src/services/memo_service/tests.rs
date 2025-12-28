use super::*;
use crate::{db, entities::user};
use chrono::Utc;
use rand::Rng;
use sea_orm::*;

async fn setup_test_db() -> (Arc<DatabaseConnection>, i32) {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL_TEST")
        .expect("DATABASE_URL_TEST must be set. Run: just setup-test-db");
    let db = Arc::new(db::create_connection(&database_url).await.unwrap());

    // 각 테스트마다 고유한 유저 생성
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
async fn test_create_and_get_memo() {
    let (db, user_id) = setup_test_db().await;
    let service = MemoService::new(db);

    let req = CreateMemoRequest {
        content: "Test memo content".to_string(),
    };

    let created = service.create_memo(user_id, req).await.unwrap();
    assert_eq!(created.content, "Test memo content");
    assert_eq!(created.user_id, user_id);
    assert!(!created.is_pinned);

    let fetched = service.get_memo(user_id, created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.content, created.content);
}

#[tokio::test]
async fn test_get_memo_unauthorized() {
    let (db, user_id) = setup_test_db().await;
    let service = MemoService::new(db);

    let req = CreateMemoRequest {
        content: "User 1's memo".to_string(),
    };

    let created = service.create_memo(user_id, req).await.unwrap();

    let result = service.get_memo(user_id + 999, created.id).await;
    assert!(matches!(result, Err(ServiceError::Unauthorized)));
}

#[tokio::test]
async fn test_update_memo() {
    let (db, user_id) = setup_test_db().await;
    let service = MemoService::new(db);

    let create_req = CreateMemoRequest {
        content: "Original content".to_string(),
    };
    let created = service.create_memo(user_id, create_req).await.unwrap();

    let update_req = UpdateMemoRequest {
        content: "Updated content".to_string(),
    };
    let updated = service
        .update_memo(user_id, created.id, update_req)
        .await
        .unwrap();

    assert_eq!(updated.content, "Updated content");
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn test_toggle_pin() {
    let (db, user_id) = setup_test_db().await;
    let service = MemoService::new(db);

    let req = CreateMemoRequest {
        content: "Pin test".to_string(),
    };
    let created = service.create_memo(user_id, req).await.unwrap();
    assert!(!created.is_pinned);

    let pinned = service.toggle_pin(user_id, created.id).await.unwrap();
    assert!(pinned.is_pinned);

    let unpinned = service.toggle_pin(user_id, created.id).await.unwrap();
    assert!(!unpinned.is_pinned);
}

#[tokio::test]
async fn test_list_memos_ordering() {
    let (db, user_id) = setup_test_db().await;
    let service = MemoService::new(db);

    let memo1 = service
        .create_memo(
            user_id,
            CreateMemoRequest {
                content: "First".to_string(),
            },
        )
        .await
        .unwrap();

    let memo2 = service
        .create_memo(
            user_id,
            CreateMemoRequest {
                content: "Second".to_string(),
            },
        )
        .await
        .unwrap();

    service.toggle_pin(user_id, memo1.id).await.unwrap();

    let memos = service.list_memos(user_id).await.unwrap();

    assert!(memos[0].is_pinned);
    assert_eq!(memos[0].id, memo1.id);
    assert!(!memos[1].is_pinned);
    assert_eq!(memos[1].id, memo2.id);
}

#[tokio::test]
async fn test_delete_memo() {
    let (db, user_id) = setup_test_db().await;
    let service = MemoService::new(db);

    let req = CreateMemoRequest {
        content: "To be deleted".to_string(),
    };
    let created = service.create_memo(user_id, req).await.unwrap();

    service.delete_memo(user_id, created.id).await.unwrap();

    let result = service.get_memo(user_id, created.id).await;
    assert!(matches!(result, Err(ServiceError::MemoNotFound)));
}
