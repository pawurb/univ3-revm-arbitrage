pub mod source;
use alloy::{
    primitives::{Bytes, U256},
    providers::{Provider, ProviderBuilder},
};
use anyhow::Result;
use revm::primitives::Bytecode;
use std::sync::Arc;
use std::{
    ops::{Div, Mul},
    str::FromStr,
};

use crate::source::{
    build_tx, custom_quoter_addr, decode_get_amount_out_response, decode_quote_response,
    get_amount_out_calldata, init_account, init_account_with_bytecode, init_cache_db,
    insert_mapping_storage_slot, me, official_quoter_addr, one_ether, pool_3000_addr,
    quote_calldata, revm_revert, usdc_addr, volumes, weth_addr,
};

#[tokio::main]
async fn main() -> Result<()> {
    let provider = ProviderBuilder::new().on_http(std::env::var("ETH_RPC_URL").unwrap().parse()?);

    let provider = Arc::new(provider);
    let base_fee = provider.get_gas_price().await?;
    let base_fee = base_fee.mul(110).div(100);

    let volumes = volumes(U256::ZERO, one_ether().div(U256::from(10)), 10);

    let mut cache_db = init_cache_db(provider.clone());

    init_account(me(), &mut cache_db, provider.clone()).await?;
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

    let mocked_custom_quoter = include_str!("bytecode/uni_v3_quoter.hex");
    let mocked_custom_quoter = Bytes::from_str(mocked_custom_quoter).unwrap();
    let mocked_custom_quoter = Bytecode::new_raw(mocked_custom_quoter);
    init_account_with_bytecode(custom_quoter_addr(), mocked_custom_quoter, &mut cache_db).await?;

    for volume in volumes {
        let call_calldata = quote_calldata(weth_addr(), usdc_addr(), volume, 3000);
        let tx = build_tx(official_quoter_addr(), me(), call_calldata, base_fee);
        let call_response = provider.call(&tx).await?;
        let call_amount_out = decode_quote_response(call_response)?;

        let revm_calldata =
            get_amount_out_calldata(pool_3000_addr(), weth_addr(), usdc_addr(), volume);
        let revm_response = revm_revert(me(), custom_quoter_addr(), revm_calldata, &mut cache_db)?;
        let revm_amount_out = decode_get_amount_out_response(revm_response)?;

        println!(
            "{} WETH -> USDC REVM {} ETH_CALL {}",
            volume, revm_amount_out, call_amount_out
        );

        if revm_amount_out != call_amount_out {
            panic!("Mismatched results!");
        }
    }

    Ok(())
}
