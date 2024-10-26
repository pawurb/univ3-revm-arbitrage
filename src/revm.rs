pub mod source;
use alloy::{primitives::U256, providers::ProviderBuilder};
use anyhow::Result;
use std::ops::Div;
use std::sync::Arc;

use crate::source::{
    decode_quote_response, init_cache_db, measure_end, measure_start, quote_calldata, revm_call,
    volumes, ME, ONE_ETHER, USDC_ADDR, V3_QUOTER_ADDR, WETH_ADDR,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);
    let provider = Arc::new(provider);

    let volumes = volumes(U256::ZERO, ONE_ETHER.div(U256::from(10)), 100);

    let mut cache_db = init_cache_db(provider.clone());

    let start = measure_start("revm_first");
    let first_volume = volumes[0];
    let calldata = quote_calldata(WETH_ADDR, USDC_ADDR, first_volume, 3000);
    let response = revm_call(ME, V3_QUOTER_ADDR, calldata, &mut cache_db)?;
    let amount_out = decode_quote_response(response)?;
    println!("{} WETH -> USDC {}", first_volume, amount_out);
    measure_end(start);

    let start = measure_start("revm");
    for (index, volume) in volumes.into_iter().enumerate() {
        let calldata = quote_calldata(WETH_ADDR, USDC_ADDR, volume, 3000);
        let response = revm_call(ME, V3_QUOTER_ADDR, calldata, &mut cache_db)?;

        let amount_out = decode_quote_response(response)?;
        if index % 20 == 0 {
            println!("{} WETH -> USDC {}", volume, amount_out);
        }
    }

    measure_end(start);

    Ok(())
}
