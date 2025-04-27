use alloy::{primitives::address, providers::ProviderBuilder};
use alloy_provider::WsConnect;
use dotenv::dotenv;
use odyssey_monitor::{
    app::run_server,
    monitor::{run_monitoring, Account, MonitorConfig},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

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

    let config = MonitorConfig::new(accounts);

    let (server, _) = run_server(&config)
        .await
        .expect("Failed to start the HTTP server");

    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new("wss://reth-ethereum.ithaca.xyz/ws"))
        .await
        .expect("could not connect to WebSocket");

    tokio::select! {
        _ = run_monitoring(config, provider) => {},
        _ = server => {},
    }

    Ok(())
}
