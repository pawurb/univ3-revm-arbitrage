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
    decode_get_amount_out_response, get_amount_out_calldata, init_account,
    init_account_with_bytecode, init_cache_db, insert_mapping_storage_slot, revm_revert, volumes,
    CUSTOM_QUOTER_ADDR, ME, ONE_ETHER, USDC_ADDR, V3_POOL_3000_ADDR, V3_POOL_500_ADDR, WETH_ADDR,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);
    let provider = Arc::new(provider);

    let volumes = volumes(U256::ZERO, ONE_ETHER.div(U256::from(10)), 100);

    let mut cache_db = init_cache_db(provider.clone());

    init_account(ME, &mut cache_db, provider.clone()).await?;
    init_account(V3_POOL_3000_ADDR, &mut cache_db, provider.clone()).await?;
    init_account(V3_POOL_500_ADDR, &mut cache_db, provider.clone()).await?;
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
    insert_mapping_storage_slot(
        WETH_ADDR,
        U256::ZERO,
        V3_POOL_500_ADDR,
        mocked_balance,
        &mut cache_db,
    )?;
    insert_mapping_storage_slot(
        USDC_ADDR,
        U256::ZERO,
        V3_POOL_500_ADDR,
        mocked_balance,
        &mut cache_db,
    )?;

    let mocked_custom_quoter = include_str!("bytecode/uni_v3_quoter.hex");
    let mocked_custom_quoter = Bytes::from_str(mocked_custom_quoter).unwrap();
    let mocked_custom_quoter = Bytecode::new_raw(mocked_custom_quoter);
    init_account_with_bytecode(CUSTOM_QUOTER_ADDR, mocked_custom_quoter, &mut cache_db)?;

    for volume in volumes.into_iter() {
        let calldata = get_amount_out_calldata(V3_POOL_500_ADDR, WETH_ADDR, USDC_ADDR, volume);
        let response = revm_revert(ME, CUSTOM_QUOTER_ADDR, calldata, &mut cache_db)?;
        let usdc_amount_out = decode_get_amount_out_response(response)?;
        let calldata = get_amount_out_calldata(
            V3_POOL_3000_ADDR,
            USDC_ADDR,
            WETH_ADDR,
            U256::from(usdc_amount_out),
        );
        let response = revm_revert(ME, CUSTOM_QUOTER_ADDR, calldata, &mut cache_db)?;
        let weth_amount_out = decode_get_amount_out_response(response)?;

        println!(
            "{} WETH -> USDC {} -> WETH {}",
            volume, usdc_amount_out, weth_amount_out
        );

        let weth_amount_out = U256::from(weth_amount_out);
        if weth_amount_out > volume {
            let profit = weth_amount_out - volume;
            println!("WETH profit: {}", profit);
        } else {
            println!("No profit.");
        }
    }

    Ok(())
}
