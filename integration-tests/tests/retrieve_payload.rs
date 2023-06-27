// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gql_client::Client;
use moat_core::{Error, PayloadRetriever};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
async fn retrieve_payload() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = Client::new(config.graphql_address.clone());

    const TXID: &str =
        "5f486c6f4edc9321e15a83993aa68463e733fc482acbde979881450c83c92a0e";

    let request: Request =
        PayloadRetriever::retrieve_tx_payload(TXID, &client).await?;

    println!("request={:?}", request);

    Ok(())
}
