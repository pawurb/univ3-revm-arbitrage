pub mod source;
use alloy::{
    primitives::{Bytes, U256},
    providers::ProviderBuilder,
};
use anyhow::Result;
use revm::state::Bytecode;
use std::sync::Arc;
use std::{ops::Div, str::FromStr};

use crate::source::{
    decode_quote_response, init_account, init_account_with_bytecode, init_cache_db,
    insert_mapping_storage_slot, measure_end, measure_start, quote_calldata, revm_call, volumes,
    ME, ONE_ETHER, USDC_ADDR, V3_POOL_3000_ADDR, V3_QUOTER_ADDR, WETH_ADDR,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);

    let provider = Arc::new(provider);

    let volumes = volumes(U256::ZERO, ONE_ETHER.div(U256::from(10)), 100);

    let mut cache_db = init_cache_db(provider.clone());

    init_account(V3_QUOTER_ADDR, &mut cache_db, provider.clone()).await?;
    init_account(V3_POOL_3000_ADDR, &mut cache_db, provider.clone()).await?;
    let mocked_erc20 = include_str!("bytecode/generic_erc20.hex");
    let mocked_erc20 = Bytes::from_str(mocked_erc20).unwrap();
    let mocked_erc20 = Bytecode::new_raw(mocked_erc20);

    init_account_with_bytecode(WETH_ADDR, mocked_erc20.clone(), &mut cache_db)?;
    init_account_with_bytecode(USDC_ADDR, mocked_erc20.clone(), &mut cache_db)?;
    let mocked_balance = U256::MAX.div(U256::from(2));
    insert_mapping_storage_slot(
        WETH_ADDR,
        U256::ZERO,
        V3_POOL_3000_ADDR,
        mocked_balance,
        &mut cache_db,
    )?;
    insert_mapping_storage_slot(
        USDC_ADDR,
        U256::ZERO,
        V3_POOL_3000_ADDR,
        mocked_balance,
        &mut cache_db,
    )?;

    let start = measure_start("revm_cached_first");
    let first_volume = volumes[0];
    let calldata = quote_calldata(WETH_ADDR, USDC_ADDR, first_volume, 3000);
    let response = revm_call(ME, V3_QUOTER_ADDR, calldata, &mut cache_db)?;
    let amount_out = decode_quote_response(response)?;
    println!("{} WETH -> USDC {}", first_volume, amount_out);
    measure_end(start);

    let start = measure_start("revm_cached");
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
