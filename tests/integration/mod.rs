use alloy::{primitives::address, providers::ProviderBuilder};
use alloy_provider::{RootProvider, WsConnect};
use alloy_pubsub::PubSubFrontend;
use odyssey_monitor::{
    app::run_server,
    monitor::{run_monitoring, Account, MonitorConfig},
};
use prometheus_parse::Scrape;
use tokio::time::Duration;

#[derive(Clone)]
struct TestApp {
    address: String,
    config: MonitorConfig,
}

impl TestApp {
    async fn new() -> Self {
        let acc_to_monitor = vec![
            Account {
                address: address!("1234562C27E07675Fe8ed90BbFB9a62853edCBb2"),
                label: "sequencer".to_string(),
            },
            Account {
                address: address!("aa52Be611a9b620aFF67FbC79326e267cc3F2c69"),
                label: "exp_er20_contract".to_string(),
            },
        ];

        let mut config = MonitorConfig::new(acc_to_monitor);
        config.app_settings.port = 0;

        let (test_server, app_address) = run_server(&config)
            .await
            .expect("Failed to start the HTTP server");
        actix_rt::spawn(test_server);

        Self {
            address: format!("http://{}", app_address),
            config,
        }
    }

    async fn create_provider(&self) -> RootProvider<PubSubFrontend> {
        ProviderBuilder::new()
            .on_ws(WsConnect::new("wss://odyssey.ithaca.xyz"))
            .await
            .expect("could not connect to WebSocket")
    }

    async fn get_metrics(&self) -> Scrape {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/metrics", self.address))
            .send()
            .await
            .expect("Failed to execute request");

        assert!(response.status().is_success());
        Scrape::parse(
            response
                .text()
                .await
                .expect("could not get response text")
                .lines()
                .map(|s| Ok(s.to_string())),
        )
        .expect("parse failed")
    }
}

#[actix_rt::test]
pub async fn health_check() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn test_metrics() {
    tracing_subscriber::fmt::init();
    let app = TestApp::new().await;

    let provider = app.create_provider().await;

    let handle = tokio::spawn(run_monitoring(app.config.clone(), provider.clone()));
    tokio::time::sleep(Duration::from_secs(5)).await;
    handle.abort();

    let scrape = app.get_metrics().await;
    let expected_metrics = vec![
        "balance_account",
        "current_block",
        "sequencer_nonce",
        "nonce_gap",
    ];

    for metric in expected_metrics {
        assert!(scrape.samples.iter().find(|s| s.metric == metric).is_some());
    }
}
