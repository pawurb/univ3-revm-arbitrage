use alloy::{
    node_bindings::Anvil,
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    transports::http::reqwest::Url,
};
use std::sync::Arc;
pub mod source;
use anyhow::Result;
use std::ops::Div;

use crate::source::{
    build_tx, decode_quote_response, measure_end, measure_start, quote_calldata, volumes, ME,
    ONE_ETHER, USDC_ADDR, V3_QUOTER_ADDR, WETH_ADDR,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let rpc_url: Url = std::env::var("ETH_RPC_URL").unwrap().parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url.clone());
    let provider = Arc::new(provider);

    let base_fee = provider.get_gas_price().await?;

    let fork_block = provider.get_block_number().await?;
    let anvil = Anvil::new()
        .fork(rpc_url)
        .fork_block_number(fork_block)
        .block_time(1_u64)
        .spawn();

    let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());
    let anvil_provider = Arc::new(anvil_provider);

    let volumes = volumes(U256::ZERO, ONE_ETHER.div(U256::from(10)), 100);

    let start = measure_start("anvil_first");
    let first_volume = volumes[0];
    let calldata = quote_calldata(WETH_ADDR, USDC_ADDR, first_volume, 3000);
    let tx = build_tx(V3_QUOTER_ADDR, ME, calldata, base_fee);
    let response = anvil_provider.call(tx).await?;
    let amount_out = decode_quote_response(response)?;
    println!("{} WETH -> USDC {}", first_volume, amount_out);
    measure_end(start);

    let start = measure_start("anvil");
    for (index, volume) in volumes.into_iter().enumerate() {
        let calldata = quote_calldata(WETH_ADDR, USDC_ADDR, volume, 3000);
        let tx = build_tx(V3_QUOTER_ADDR, ME, calldata, base_fee);
        let response = anvil_provider.call(tx).await?;
        let amount_out = decode_quote_response(response)?;
        if index % 20 == 0 {
            println!("{} WETH -> USDC {}", volume, amount_out);
        }
    }

    measure_end(start);
    drop(anvil);

    Ok(())
}
