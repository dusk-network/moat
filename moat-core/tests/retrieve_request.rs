// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{Error, RequestRetriever};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
async fn retrieve_request() -> Result<(), Error> {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    // RequestRetriever::retrieve_transaction(&blockchain_access_config).await?;
    RequestRetriever::retrieve_block(&blockchain_access_config).await?;

    Ok(())
}
