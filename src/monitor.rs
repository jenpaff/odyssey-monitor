use crate::{app::AppSettings, BALANCE_ACCOUNT};
use crate::{CURRENT_BLOCK, SEQUENCER_NONCE, SEQUENCER_NONCE_GAP};
use alloy::network::Ethereum;
use alloy::primitives::utils::format_units;
use alloy::primitives::Address;
use alloy::providers::Provider;
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

pub async fn run_monitoring<P>(config: MonitorConfig, provider: P) -> Result<()>
where
    P: Provider<Ethereum>,
{
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
            block.number,
            block.hash
        );

        CURRENT_BLOCK.set(block.number as i64);

        let nonce_fut = provider
            .get_transaction_count(sequencer)
            .block_id(block.number.into())
            .into_future();

        let pending_nonce_fut = provider
            .get_transaction_count(sequencer)
            .pending()
            .into_future();

        let balances_fut = config
            .accounts
            .iter()
            .map(|a| {
                provider
                    .get_balance(a.address)
                    .block_id(block.number.into())
                    .into_future()
            })
            .collect::<Vec<_>>();

        let (sequencer_nonce, sequencer_nonce_pending, balances) =
            join!(nonce_fut, pending_nonce_fut, join_all(balances_fut));

        if let (Ok(sequencer_nonce), Ok(pending_nonce)) = (sequencer_nonce, sequencer_nonce_pending)
        {
            tracing::info!("#️⃣  Sequencer Nonce : {:?}", sequencer_nonce);
            SEQUENCER_NONCE.set(sequencer_nonce as i64);

            let nonce_gap = pending_nonce.saturating_sub(sequencer_nonce);
            SEQUENCER_NONCE_GAP.set(nonce_gap as i64);
            tracing::info!("📨 Sequencer nonce gap: {:?}", nonce_gap,);
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
