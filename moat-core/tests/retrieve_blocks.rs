// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gql_client::Client;
use moat_core::{Error, RequestRetriever};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
async fn retrieve_block() -> Result<(), Error> {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let cfg =
        BlockchainAccessConfig::load_path(config_path)?;

    let client = Client::new(cfg.graphql_address.clone());

    const BLOCK_HEIGHT: u64 = 97117;

    let txs = RequestRetriever::retrieve_txs_from_block(&client, BLOCK_HEIGHT).await?;

    assert_eq!(txs.transactions.len(), 1);

    println!("transactions={:?}", txs);

    Ok(())
}
