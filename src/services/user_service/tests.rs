use super::*;
use crate::entities::oauth_account::OAuthProvider;

async fn setup_test_db() -> Arc<DatabaseConnection> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL_TEST")
        .expect("DATABASE_URL_TEST must be set. Run: just setup-test-db");
    Arc::new(crate::db::create_connection(&database_url).await.unwrap())
}

#[tokio::test]
async fn test_oauth_login_new_user() {
    let db = setup_test_db().await;
    let service = UserService::new(db);

    let req = OAuthLoginRequest {
        provider: OAuthProvider::Google,
        provider_user_id: "google_123".to_string(),
        email: "newuser@example.com".to_string(),
        username: "newuser".to_string(),
    };

    let result = service.oauth_login(req).await.unwrap();

    assert_eq!(result.username, "newuser");
    assert_eq!(result.email, "newuser@example.com");
}

#[tokio::test]
async fn test_oauth_login_existing_oauth_account() {
    let db = setup_test_db().await;
    let service = UserService::new(db);

    let req = OAuthLoginRequest {
        provider: OAuthProvider::Kakao,
        provider_user_id: "kakao_456".to_string(),
        email: "existing@example.com".to_string(),
        username: "existing".to_string(),
    };

    let first_login = service.oauth_login(req.clone()).await.unwrap();
    let second_login = service.oauth_login(req).await.unwrap();

    assert_eq!(first_login.id, second_login.id);
    assert_eq!(first_login.email, second_login.email);
}

#[tokio::test]
async fn test_oauth_login_different_provider_same_email() {
    let db = setup_test_db().await;
    let service = UserService::new(db);

    let google_req = OAuthLoginRequest {
        provider: OAuthProvider::Google,
        provider_user_id: "google_789".to_string(),
        email: "multiauth@example.com".to_string(),
        username: "multiauth".to_string(),
    };

    let kakao_req = OAuthLoginRequest {
        provider: OAuthProvider::Kakao,
        provider_user_id: "kakao_789".to_string(),
        email: "multiauth@example.com".to_string(),
        username: "multiauth".to_string(),
    };

    let google_login = service.oauth_login(google_req).await.unwrap();
    let kakao_login = service.oauth_login(kakao_req).await.unwrap();

    assert_eq!(google_login.id, kakao_login.id);
}
