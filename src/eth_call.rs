use alloy::{
    primitives::U256,
    providers::{Provider, ProviderBuilder},
};
use std::sync::Arc;
pub mod source;
use anyhow::Result;
use std::ops::Div;

use crate::source::{
    build_tx, decode_quote_response, me, measure_end, measure_start, official_quoter_addr,
    one_ether, quote_calldata, usdc_addr, volumes, weth_addr,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);
    let provider = Arc::new(provider);

    let base_fee = provider.get_gas_price().await?;

    let volumes = volumes(U256::ZERO, one_ether().div(U256::from(10)), 100);

    let start = measure_start("eth_call");
    for (index, volume) in volumes.into_iter().enumerate() {
        let calldata = quote_calldata(weth_addr(), usdc_addr(), volume, 3000);
        let tx = build_tx(official_quoter_addr(), me(), calldata, base_fee);
        let response = provider.call(&tx).await?;
        let amount_out = decode_quote_response(response)?;
        if index % 20 == 0 {
            println!("{} WETH -> USDC {}", volume, amount_out);
        }
    }

    measure_end(start);

    Ok(())
}
