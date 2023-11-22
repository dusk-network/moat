// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::RuskHttpClient;
use moat::wallet_accessor::BlockchainAccessConfig;
use moat::{Error, TxInquirer};
use toml_base_config::BaseConfig;
use tracing::trace;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn retrieve_txs_from_block() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(cfg.rusk_address);

    const BLOCK_HEIGHT: u64 = 110;

    let txs = TxInquirer::txs_from_block(&client, BLOCK_HEIGHT).await?;

    trace!("transactions retrieved={}", txs.transactions.len());

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

    let (txs, top_block) = TxInquirer::txs_from_block_range(
        &client,
        BLOCK_HEIGHT_BEG,
        BLOCK_HEIGHT_END,
    )
    .await?;

    trace!("transactions retrieved={}", txs.transactions.len());
    trace!("current top block={}", top_block);

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
    let txs = TxInquirer::txs_from_last_n_blocks(&client, N).await?;

    trace!("transactions={}", txs.transactions.len());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "vol_tests"), ignore)]
async fn retrieve_tx_by_id() -> Result<(), Error> {
    const TXID: &str =
        "44fe2c6407fc400a2dee6e30c62a02b82f3980da18d3b6306e80f9f83730520d";

    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let (tx, height) = TxInquirer::retrieve_tx(TXID, &client).await?;

    trace!("tx={:?}, block_height={}", tx, height);

    Ok(())
}
