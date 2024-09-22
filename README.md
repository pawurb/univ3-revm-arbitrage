# Uniswap V3 MEV arbitrage calculations with REVM

Examples for [https://pawelurbanek.com/revm-alloy-anvil-arbitrage](https://pawelurbanek.com/revm-alloy-anvil-arbitrage)

Usage:

```bash
source .env && cargo run --bin eth_call_one --release
source .env && cargo run --bin eth_call --release
source .env && cargo run --bin anvil --release
source .env && cargo run --bin revm --release
source .env && cargo run --bin revm_cached --release
source .env && cargo run --bin revm_quoter --release
source .env && cargo run --bin revm_validate --release
source .env && cargo run --bin revm_arbitrage --release
```


