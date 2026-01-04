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
    let service = UserService::new(db).expect("Failed to create UserService");

    let req = OAuthLoginRequest {
        provider: OAuthProvider::Google,
        provider_user_id: "google_123".to_string(),
        email: "newuser@example.com".to_string(),
        username: "newuser".to_string(),
    };

    let (auth_response, access_token, refresh_token) = service.oauth_login(req).await.unwrap();

    assert_eq!(auth_response.user.username, "newuser");
    assert_eq!(auth_response.user.email, "newuser@example.com");
    assert!(access_token.len() > 0);
    assert!(refresh_token.len() > 0);
}

#[tokio::test]
async fn test_oauth_login_existing_oauth_account() {
    let db = setup_test_db().await;
    let service = UserService::new(db).expect("Failed to create UserService");

    let req = OAuthLoginRequest {
        provider: OAuthProvider::Kakao,
        provider_user_id: "kakao_456".to_string(),
        email: "existing@example.com".to_string(),
        username: "existing".to_string(),
    };

    let (first_auth, first_access, first_refresh) = service.oauth_login(req.clone()).await.unwrap();
    let (second_auth, second_access, second_refresh) = service.oauth_login(req).await.unwrap();

    assert_eq!(first_auth.user.id, second_auth.user.id);
    assert_eq!(first_auth.user.email, second_auth.user.email);
    assert!(first_access.len() > 0);
    assert!(second_access.len() > 0);
    assert!(first_refresh.len() > 0);
    assert!(second_refresh.len() > 0);
}

#[tokio::test]
async fn test_oauth_login_different_provider_same_email() {
    let db = setup_test_db().await;
    let service = UserService::new(db).expect("Failed to create UserService");

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

    let (google_auth, google_access, google_refresh) = service.oauth_login(google_req).await.unwrap();
    let (kakao_auth, kakao_access, kakao_refresh) = service.oauth_login(kakao_req).await.unwrap();

    assert_eq!(google_auth.user.id, kakao_auth.user.id);
    assert!(google_access.len() > 0);
    assert!(kakao_access.len() > 0);
    assert!(google_refresh.len() > 0);
    assert!(kakao_refresh.len() > 0);
}
