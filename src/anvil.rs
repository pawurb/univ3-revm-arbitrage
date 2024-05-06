use alloy_provider::{Provider, ProviderBuilder};
use reqwest::Url;
use std::sync::Arc;
pub mod source;
use alloy_node_bindings::Anvil;
use anyhow::Result;
use revm::primitives::U256;
use std::ops::Div;

use crate::source::{
    build_tx, decode_quote_response, me, measure_end, measure_start, official_quoter_addr,
    one_ether, quote_calldata, usdc_addr, volumes, weth_addr,
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
        .port(portpicker::pick_unused_port().expect("No ports free for Anvil") as u16)
        .fork_block_number(fork_block)
        .block_time(1_u64)
        .spawn();

    let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse().unwrap());
    let anvil_provider = Arc::new(anvil_provider);

    let volumes = volumes(U256::from(0), one_ether().div(U256::from(10)), 100);

    let start = measure_start("anvil_first");
    let first_volume = volumes[0];
    let calldata = quote_calldata(weth_addr(), usdc_addr(), first_volume, 3000);
    let tx = build_tx(official_quoter_addr(), me(), calldata, base_fee);
    let response = anvil_provider.call(&tx).await?;
    let amount_out = decode_quote_response(response)?;
    println!("{} WETH -> USDC {}", first_volume, amount_out);
    measure_end(start);

    let start = measure_start("anvil");
    for (index, volume) in volumes.into_iter().enumerate() {
        let calldata = quote_calldata(weth_addr(), usdc_addr(), volume, 3000);
        let tx = build_tx(official_quoter_addr(), me(), calldata, base_fee);
        let response = anvil_provider.call(&tx).await?;
        let amount_out = decode_quote_response(response)?;
        if index % 20 == 0 {
            println!("{} WETH -> USDC {}", volume, amount_out);
        }
    }

    measure_end(start);
    drop(anvil);

    Ok(())
}
