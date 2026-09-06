use rooney::ai::OllamaClient;

#[tokio::test]
async fn test_ollama_connectivity_and_models() {
    let mut client = OllamaClient::default();
    let result = client.fetch_models().await;

    match result {
        Ok(models) => {
            println!("Ollama models found: {:?}", models);
            assert!(!models.is_empty(), "Expected at least one model in Ollama");
        }
        Err(e) => {
            println!("Ollama not running or unreachable: {}", e);
        }
    }
}

#[tokio::test]
async fn test_ollama_fim_generation() {
    let mut client = OllamaClient::default();
    let _ = client.fetch_models().await;

    if client.available_models.iter().any(|m| m.contains("deepseek")) {
        client.active_model = "deepseek-coder-v2:16b".to_string();
        let prefix = "fn calculate_area(width: f64, height: f64) -> f64 {\n    ";
        let suffix = "\n}\n";

        let completion = client.generate_fim(prefix, suffix).await;
        println!("FIM generated: {:?}", completion);
        assert!(completion.is_ok(), "FIM completion should succeed");
    }
}
