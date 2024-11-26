# Odyssey Monitor

Simple prometheus exporter to fetch metrics and configure alerts for Odyssey Testnet. 

## Exported Prometheus metrics

| name                                | description                                                                             |
| ----------------------------------- | ----------------------------------------------------------------------------------      |
| `balance_account`                   | Exposes balance of ERC20 contract used for experiments and sequencer                    |
| `sequencer_nonce`                   | Exposes nonce of the sequencer                                                          |
| `current_block`                     | Exposes current block height                                                            |
| `sequencer_nonce`, `nonce_gap`      | Exposes current sequencer none & nonce gap to alert if sequencer is stuck               |

## Build & Test

```rust
cargo run
cargo test
```

## Run & Test with Docker

To run odyssey-monitor with prometheus, grafana & alert manager locally, run with Docker:

```bash
docker compose up -d
```
