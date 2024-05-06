pub mod source;
use alloy_provider::ProviderBuilder;
use anyhow::Result;
use revm::primitives::{Bytecode, Bytes, U256};
use std::sync::Arc;
use std::{ops::Div, str::FromStr};

use crate::source::{
    custom_quoter_addr, decode_get_amount_out_response, get_amount_out_calldata, init_account,
    init_account_with_bytecode, init_cache_db, insert_mapping_storage_slot, me, one_ether,
    pool_3000_addr, pool_500_addr, revm_revert, usdc_addr, volumes, weth_addr,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);
    let provider = Arc::new(provider);

    let volumes = volumes(U256::from(0), one_ether().div(U256::from(10)), 100);

    let mut cache_db = init_cache_db(provider.clone());

    init_account(me(), &mut cache_db, provider.clone()).await?;
    init_account(pool_3000_addr(), &mut cache_db, provider.clone()).await?;
    init_account(pool_500_addr(), &mut cache_db, provider.clone()).await?;
    let mocked_erc20 = include_str!("bytecode/generic_erc20.hex");
    let mocked_erc20 = Bytes::from_str(mocked_erc20).unwrap();
    let mocked_erc20 = Bytecode::new_raw(mocked_erc20);

    init_account_with_bytecode(weth_addr(), mocked_erc20.clone(), &mut cache_db).await?;
    init_account_with_bytecode(usdc_addr(), mocked_erc20.clone(), &mut cache_db).await?;

    let mocked_balance = U256::MAX.div(U256::from(2));
    insert_mapping_storage_slot(
        weth_addr(),
        U256::from(0),
        pool_3000_addr(),
        mocked_balance,
        &mut cache_db,
    )
    .await?;
    insert_mapping_storage_slot(
        usdc_addr(),
        U256::from(0),
        pool_3000_addr(),
        mocked_balance,
        &mut cache_db,
    )
    .await?;
    insert_mapping_storage_slot(
        weth_addr(),
        U256::from(0),
        pool_500_addr(),
        mocked_balance,
        &mut cache_db,
    )
    .await?;
    insert_mapping_storage_slot(
        usdc_addr(),
        U256::from(0),
        pool_500_addr(),
        mocked_balance,
        &mut cache_db,
    )
    .await?;

    let mocked_custom_quoter = include_str!("bytecode/uni_v3_quoter.hex");
    let mocked_custom_quoter = Bytes::from_str(mocked_custom_quoter).unwrap();
    let mocked_custom_quoter = Bytecode::new_raw(mocked_custom_quoter);
    init_account_with_bytecode(custom_quoter_addr(), mocked_custom_quoter, &mut cache_db).await?;

    for volume in volumes.into_iter() {
        let calldata = get_amount_out_calldata(pool_500_addr(), weth_addr(), usdc_addr(), volume);
        let response = revm_revert(me(), custom_quoter_addr(), calldata, &mut cache_db)?;
        let usdc_amount_out = decode_get_amount_out_response(response)?;
        let calldata = get_amount_out_calldata(
            pool_3000_addr(),
            usdc_addr(),
            weth_addr(),
            U256::from(usdc_amount_out),
        );
        let response = revm_revert(me(), custom_quoter_addr(), calldata, &mut cache_db)?;
        let weth_amount_out = decode_get_amount_out_response(response)?;

        println!(
            "{} WETH -> USDC {} -> WETH {}",
            volume, usdc_amount_out, weth_amount_out
        );

        let weth_amount_out = U256::from(volume);
        if weth_amount_out > volume {
            let profit = weth_amount_out - volume;
            println!("WETH profit: {}", profit);
        } else {
            println!("No profit.");
        }
    }

    Ok(())
}
