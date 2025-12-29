use super::*;

#[tokio::test]
#[ignore]
async fn test_real_gemini_embedding() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    let client = GeminiClient::new(api_key);

    let result = client.embed("안녕하세요").await;

    assert!(result.is_ok(), "Embedding failed: {:?}", result.err());
    let vector = result.unwrap();
    assert_eq!(
        vector.len(),
        768,
        "Expected 768 dimensions, got {}",
        vector.len()
    );
    assert_eq!(client.dimension(), 768);

    println!("✅ Gemini Embedding API 연결 성공!");
    println!("   벡터 차원: {}", vector.len());
    println!("   벡터 샘플: {:?}", &vector[0..5]);
}

#[tokio::test]
#[ignore]
async fn test_real_gemini_generation() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    let client = GeminiClient::new(api_key);

    let result = client
        .generate(
            "사랑에 대해 쓰고 싶어",
            vec!["사랑은 수용이다".to_string()],
        )
        .await;

    assert!(
        result.is_ok(),
        "Text generation failed: {:?}",
        result.err()
    );
    let text = result.unwrap();
    assert!(!text.is_empty(), "Generated text is empty");

    println!("✅ Gemini Text Generation API 연결 성공!");
    println!("   생성된 텍스트 길이: {} bytes", text.len());
    println!("   생성된 텍스트:\n{}", text);
}
