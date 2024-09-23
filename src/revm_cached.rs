pub mod source;
use alloy::{
    primitives::{Bytes, U256},
    providers::ProviderBuilder,
};
use anyhow::Result;
use revm::primitives::Bytecode;
use std::sync::Arc;
use std::{ops::Div, str::FromStr};

use crate::source::{
    decode_quote_response, init_account, init_account_with_bytecode, init_cache_db,
    insert_mapping_storage_slot, me, measure_end, measure_start, official_quoter_addr, one_ether,
    pool_3000_addr, quote_calldata, revm_call, usdc_addr, volumes, weth_addr,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);

    let provider = Arc::new(provider);

    let volumes = volumes(U256::ZERO, one_ether().div(U256::from(10)), 100);

    let mut cache_db = init_cache_db(provider.clone());

    init_account(official_quoter_addr(), &mut cache_db, provider.clone()).await?;
    init_account(pool_3000_addr(), &mut cache_db, provider.clone()).await?;
    let mocked_erc20 = include_str!("bytecode/generic_erc20.hex");
    let mocked_erc20 = Bytes::from_str(mocked_erc20).unwrap();
    let mocked_erc20 = Bytecode::new_raw(mocked_erc20);

    init_account_with_bytecode(weth_addr(), mocked_erc20.clone(), &mut cache_db).await?;
    init_account_with_bytecode(usdc_addr(), mocked_erc20.clone(), &mut cache_db).await?;
    let mocked_balance = U256::MAX.div(U256::from(2));
    insert_mapping_storage_slot(
        weth_addr(),
        U256::ZERO,
        pool_3000_addr(),
        mocked_balance,
        &mut cache_db,
    )
    .await?;
    insert_mapping_storage_slot(
        usdc_addr(),
        U256::ZERO,
        pool_3000_addr(),
        mocked_balance,
        &mut cache_db,
    )
    .await?;

    let start = measure_start("revm_cached_first");
    let first_volume = volumes[0];
    let calldata = quote_calldata(weth_addr(), usdc_addr(), first_volume, 3000);
    let response = revm_call(me(), official_quoter_addr(), calldata, &mut cache_db)?;
    let amount_out = decode_quote_response(response)?;
    println!("{} WETH -> USDC {}", first_volume, amount_out);
    measure_end(start);

    let start = measure_start("revm_cached");
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
