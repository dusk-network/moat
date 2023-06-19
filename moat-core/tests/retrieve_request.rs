// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{Error, RequestRetriever};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

#[tokio::test(flavor = "multi_thread")]
async fn retrieve_request() -> Result<(), Error> {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    const TXID: &str =
        "ce90bb9d95192668cfe240d4eea0574bfdf1e7cdfe800f53a034077d2b55dc01";

    let _: Request = RequestRetriever::retrieve_tx_payload(
        TXID,
        &blockchain_access_config,
    )
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn retrieve_block() -> Result<(), Error> {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    RequestRetriever::retrieve_block(&blockchain_access_config).await?;

    Ok(())
}
