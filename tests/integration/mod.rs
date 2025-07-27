use alloy::{
    primitives::address,
    providers::ProviderBuilder,
};
use alloy_provider::WsConnect;
use odyssey_monitor::{
    app::run_server,
    monitor::{run_monitoring, Account, MonitorConfig},
};
use prometheus_parse::Scrape;
use tokio::time::Duration;

#[actix_rt::test]
async fn test_metrics_endpoint() {
    tracing_subscriber::fmt::init();

    // Create config matching main.rs pattern
    let accounts = vec![
        Account {
            address: address!("1234562C27E07675Fe8ed90BbFB9a62853edCBb2"),
            label: "sequencer".to_string(),
        },
        Account {
            address: address!("238c8CD93ee9F8c7Edf395548eF60c0d2e46665E"),
            label: "exp_erc20_contract".to_string(),
        },
    ];

    let mut config = MonitorConfig::new(accounts);
    config.app_settings.port = 0; // Use random port for testing

    // Start the HTTP server
    let (server, server_address) = run_server(&config)
        .await
        .expect("Failed to start the HTTP server");

    // Create provider matching main.rs
    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new("wss://odyssey.ithaca.xyz"))
        .await
        .expect("could not connect to WebSocket");

    // Run monitoring in the background
    let monitoring_handle = tokio::spawn(run_monitoring(config.clone(), provider));
    
    // Run server in the background
    let server_handle = tokio::spawn(server);

    // Wait for monitoring to collect some data
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Fetch metrics from the integrated app
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/metrics", server_address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    
    let metrics_text = response
        .text()
        .await
        .expect("could not get response text");
    
    let scrape = Scrape::parse(
        metrics_text
            .lines()
            .map(|s| Ok(s.to_string())),
    )
    .expect("parse failed");

    // Verify metrics exist and have non-zero values
    let balance_metrics: Vec<_> = scrape.samples.iter()
        .filter(|s| s.metric == "balance_account")
        .collect();
    assert!(!balance_metrics.is_empty(), "balance_account metrics should exist");
    
    let current_block = scrape.samples.iter()
        .find(|s| s.metric == "current_block");
    assert!(current_block.is_some(), "current_block metric should exist");
    match &current_block.unwrap().value {
        prometheus_parse::Value::Gauge(v) | prometheus_parse::Value::Counter(v) | prometheus_parse::Value::Untyped(v) => {
            assert!(*v > 0.0, "current_block should be greater than 0");
        }
        _ => panic!("current_block should be a gauge, counter, or untyped metric"),
    }
    
    let sequencer_nonce = scrape.samples.iter()
        .find(|s| s.metric == "sequencer_nonce");
    assert!(sequencer_nonce.is_some(), "sequencer_nonce metric should exist");
    
    // Clean up
    monitoring_handle.abort();
    server_handle.abort();
}

#[actix_rt::test]
async fn test_health_endpoint() {
    let accounts = vec![];
    let mut config = MonitorConfig::new(accounts);
    config.app_settings.port = 0;

    let (server, server_address) = run_server(&config)
        .await
        .expect("Failed to start the HTTP server");
    
    let server_handle = tokio::spawn(server);

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/health", server_address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    
    server_handle.abort();
}