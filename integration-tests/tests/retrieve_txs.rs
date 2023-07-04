// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gql_client::Client;
use moat_core::{Error, TxRetriever};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_block() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = Client::new(cfg.graphql_address.clone());

    const BLOCK_HEIGHT: u64 = 317042;

    let txs = TxRetriever::txs_from_block(&client, BLOCK_HEIGHT).await?;

    println!("transactions={:?}", txs);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_block_range() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = Client::new(cfg.graphql_address.clone());

    const BLOCK_HEIGHT_BEG: u64 = 97117;
    const BLOCK_HEIGHT_END: u64 = 107117;

    let (txs, top_block) = TxRetriever::txs_from_block_range(
        &client,
        BLOCK_HEIGHT_BEG,
        BLOCK_HEIGHT_END,
    )
    .await?;

    assert!(top_block > 0);

    println!("transactions={:?}", txs);
    println!("current top block={}", top_block);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_last_n_blocks() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = Client::new(cfg.graphql_address.clone());

    const N: u32 = 10000;
    let txs = TxRetriever::txs_from_last_n_blocks(&client, N).await?;

    println!("transactions={:?}", txs);

    Ok(())
}
