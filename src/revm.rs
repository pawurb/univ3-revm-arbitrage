pub mod source;
use alloy::{primitives::U256, providers::ProviderBuilder};
use anyhow::Result;
use std::ops::Div;
use std::sync::Arc;

use crate::source::{
    decode_quote_response, init_cache_db, me, measure_end, measure_start, official_quoter_addr,
    one_ether, quote_calldata, revm_call, usdc_addr, volumes, weth_addr,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);
    let provider = Arc::new(provider);

    let volumes = volumes(U256::from(0), one_ether().div(U256::from(10)), 100);

    let mut cache_db = init_cache_db(provider.clone());

    let start = measure_start("revm_first");
    let first_volume = volumes[0];
    let calldata = quote_calldata(weth_addr(), usdc_addr(), first_volume, 3000);
    let response = revm_call(me(), official_quoter_addr(), calldata, &mut cache_db)?;
    let amount_out = decode_quote_response(response)?;
    println!("{} WETH -> USDC {}", first_volume, amount_out);
    measure_end(start);

    let start = measure_start("revm");
    for (index, volume) in volumes.into_iter().enumerate() {
        let calldata = quote_calldata(weth_addr(), usdc_addr(), volume, 3000);
        let response = revm_call(me(), official_quoter_addr(), calldata, &mut cache_db)?;

        let amount_out = decode_quote_response(response)?;
        if index % 20 == 0 {
            println!("{} WETH -> USDC {}", volume, amount_out);
        }
    }

    measure_end(start);

    Ok(())
}
