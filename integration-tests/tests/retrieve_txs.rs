// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::RuskHttpClient;
use moat_core::{Error, TxRetriever};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_block() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(cfg.rusk_address);

    const BLOCK_HEIGHT: u64 = 110;

    let txs = TxRetriever::txs_from_block(&client, BLOCK_HEIGHT).await?;

    println!("transactions retrieved={}", txs.transactions.len());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_block_range() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(cfg.rusk_address);

    const BLOCK_HEIGHT_BEG: u64 = 1;
    const BLOCK_HEIGHT_END: u64 = 1000;

    let (txs, top_block) = TxRetriever::txs_from_block_range(
        &client,
        BLOCK_HEIGHT_BEG,
        BLOCK_HEIGHT_END,
    )
    .await?;

    assert!(top_block > 0);

    println!("transactions retrieved={}", txs.transactions.len());
    println!("current top block={}", top_block);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_last_n_blocks() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(cfg.rusk_address);

    const N: usize = 10000;
    let txs = TxRetriever::txs_from_last_n_blocks(&client, N).await?;

    println!("transactions={}", txs.transactions.len());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "vol_tests"), ignore)]
async fn retrieve_tx_by_id() -> Result<(), Error> {
    const TXID: &str =
        "b71919ccaf9cd15592d19edd5c3bd164ccd95ab33e92b7e06cb4d6065142e401";

    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let result = TxRetriever::retrieve_tx(TXID, &client).await;

    println!("res={:?}", result);

    assert!(result.is_ok());

    Ok(())
}
