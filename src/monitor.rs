use crate::{app::AppSettings, BALANCE_ACCOUNT};
use crate::{CURRENT_BLOCK, SEQUENCER_NONCE};
use alloy::primitives::utils::format_units;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy_provider::RootProvider;
use alloy_pubsub::PubSubFrontend;
use anyhow::Result;
use futures::{future::join_all, StreamExt};
use std::env;
use std::future::IntoFuture;
use tokio::join;

#[derive(Debug, Clone)]
pub struct Account {
    pub address: Address,
    pub label: String,
}

#[derive(Clone)]
pub struct MonitorConfig {
    pub rpc_url: String,
    pub app_settings: AppSettings,
    pub accounts: Vec<Account>,
}

impl MonitorConfig {
    pub fn new(accounts: Vec<Account>) -> Self {
        let rpc_url = env::var("RPC_URL").unwrap_or("https://odyssey.ithaca.xyz".to_string());

        Self {
            rpc_url,
            app_settings: AppSettings::default(),
            accounts,
        }
    }
}

pub async fn run_monitoring(
    config: MonitorConfig,
    provider: RootProvider<PubSubFrontend>,
) -> Result<()> {
    let sequencer = config
        .accounts
        .iter()
        .find(|a| a.label == "sequencer")
        .unwrap()
        .address;

    let mut block_stream = provider.subscribe_blocks().await?.into_stream();

    while let Some(block) = block_stream.next().await {
        tracing::info!(
            "🧱 New block detected : Block Number : {:?} Block Hash: {:?}",
            block.header.number,
            block.header.hash
        );

        CURRENT_BLOCK.set(block.header.number as i64);

        let nonce_fut = provider
            .get_transaction_count(sequencer)
            .block_id(block.header.number.into())
            .into_future();

        let balances_fut = config
            .accounts
            .iter()
            .map(|a| {
                provider
                    .get_balance(a.address)
                    .block_id(block.header.number.into())
                    .into_future()
            })
            .collect::<Vec<_>>();

        let (sequencer_nonce, balances) = join!(nonce_fut, join_all(balances_fut));

        if let Ok(sequencer_nonce) = sequencer_nonce {
            tracing::info!("#️⃣  Sequencer Nonce : {:?}", sequencer_nonce);
            SEQUENCER_NONCE.set(sequencer_nonce as i64);
        }

        for (account, balance) in config.accounts.iter().zip(balances) {
            if let Ok(balance) = balance {
                if let Ok(eth_balance) = format_units(balance, 18).unwrap().parse::<f64>() {
                    tracing::info!("💰 Balance for {}: {} ETH", account.label, eth_balance);
                    BALANCE_ACCOUNT
                        .with_label_values(&[&account.address.to_string(), &account.label])
                        .set(eth_balance);
                }
            }
        }
    }

    Ok(())
}
