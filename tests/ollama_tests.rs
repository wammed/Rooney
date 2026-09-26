use rooney::ai::{ChatStreamEvent, OllamaClient};

fn is_ollama_integration_required() -> bool {
    std::env::var("OLLAMA_INTEGRATION_TEST")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

async fn get_client_and_models() -> Option<(OllamaClient, Vec<String>)> {
    let mut client = if let Ok(endpoint) = std::env::var("OLLAMA_ENDPOINT") {
        OllamaClient::new(&endpoint)
    } else {
        OllamaClient::default()
    };
    match client.fetch_models().await {
        Ok(models) => Some((client, models)),
        Err(e) => {
            if is_ollama_integration_required() {
                panic!(
                    "Ollama integration test failed: server at {} is unreachable: {}",
                    client.endpoint, e
                );
            } else {
                eprintln!(
                    "Skipping Ollama test: server at {} is unreachable: {}. (Set OLLAMA_INTEGRATION_TEST=1 to enforce)",
                    client.endpoint, e
                );
                None
            }
        }
    }
}

#[tokio::test]
async fn test_ollama_connectivity_and_models() {
    let Some((_, models)) = get_client_and_models().await else {
        return;
    };

    println!("Ollama models found: {:?}", models);
    if is_ollama_integration_required() {
        assert!(
            !models.is_empty(),
            "Expected at least one model installed in Ollama for integration test"
        );
    }
}

#[tokio::test]
async fn test_ollama_fim_generation() {
    let Some((mut client, models)) = get_client_and_models().await else {
        return;
    };

    if models.is_empty() {
        if is_ollama_integration_required() {
            panic!("Ollama integration test failed: no models available in Ollama");
        } else {
            eprintln!("Skipping test_ollama_fim_generation: no models installed in Ollama");
            return;
        }
    }

    // Select the best available model for FIM among the discovered models.
    // Prefer coder/deepseek models that support FIM prompt formatting.
    let target_model = models
        .iter()
        .find(|m| m.contains("coder") || m.contains("code") || m.contains("deepseek"))
        .cloned()
        .or_else(|| models.first().cloned());

    let Some(model_name) = target_model else {
        println!(
            "Skipping FIM generation: no suitable model found among installed models: {:?}",
            models
        );
        return;
    };

    client.active_model = model_name.clone();
    println!(
        "Testing FIM generation using active model: {}",
        client.active_model
    );

    let prefix = "fn calculate_area(width: f64, height: f64) -> f64 {\n    ";
    let suffix = "\n}\n";

    let completion = client.generate_fim(prefix, suffix).await;
    println!("FIM generated: {:?}", completion);
    assert!(
        completion.is_ok(),
        "FIM completion failed for model {}: {:?}",
        model_name,
        completion.err()
    );
}

#[tokio::test]
async fn test_ollama_chat_streaming() {
    let Some((mut client, models)) = get_client_and_models().await else {
        return;
    };

    if models.is_empty() {
        if is_ollama_integration_required() {
            panic!("Ollama integration test failed: no models available in Ollama");
        } else {
            eprintln!("Skipping test_ollama_chat_streaming: no models installed in Ollama");
            return;
        }
    }

    // Use the first available discovered model
    client.active_model = models[0].clone();
    let model_name = client.active_model.clone();
    println!("Testing chat streaming using active model: {}", model_name);

    let (tx, mut rx) = futures_channel::mpsc::unbounded::<ChatStreamEvent>();
    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    tokio::spawn(async move {
        client
            .chat_generate_stream(None, "Say hello in one word", tx, cancel_flag)
            .await;
    });

    use futures_util::StreamExt;
    let mut chunks = Vec::new();
    let mut got_done = false;
    let mut stream_error = None;

    while let Some(event) = rx.next().await {
        match event {
            ChatStreamEvent::Chunk(c) => chunks.push(c),
            ChatStreamEvent::Done => {
                got_done = true;
                break;
            }
            ChatStreamEvent::Error(e) => {
                stream_error = Some(e);
                break;
            }
        }
    }

    println!("Streaming chunks received: {:?}", chunks);
    if let Some(err) = stream_error {
        panic!("Chat streaming failed for model {}: {}", model_name, err);
    }
    assert!(got_done, "Streaming should complete with Done event");
    assert!(
        !chunks.is_empty(),
        "Streaming should yield at least one chunk"
    );
}
