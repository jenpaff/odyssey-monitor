pub mod app;
pub mod monitor;

use lazy_static::lazy_static;
use prometheus::{register_gauge_vec, register_int_gauge, GaugeVec, IntGauge};

lazy_static! {
    pub static ref BALANCE_ACCOUNT: GaugeVec = register_gauge_vec!(
        "balance_account",
        "balances for accounts configured",
        &["address", "label"],
    )
    .expect("Cannot create balance account metric");
    pub static ref CURRENT_BLOCK: IntGauge =
        register_int_gauge!("current_block", "Processing current block")
            .expect("Cannot create block metric");
    pub static ref SEQUENCER_NONCE: IntGauge =
        register_int_gauge!("sequencer_nonce", "Current sequencer nonce")
            .expect("Cannot create nonce metric");
    pub static ref SEQUENCER_NONCE_GAP: IntGauge =
        register_int_gauge!("nonce_gap", "Sequencer nonce gap")
            .expect("Cannot create nonce gap metric");
}
