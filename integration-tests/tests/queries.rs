// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::ops::Range;
use moat_core::Error;
use dusk_wallet::RuskHttpClient;
use rkyv::{check_archived_root, Deserialize, Infallible};
use toml_base_config::BaseConfig;
use moat_core::Error::InvalidQueryResponse;
use wallet_accessor::BlockchainAccessConfig;

const LICENSE_CONTRACT: &str =
    "0300000000000000000000000000000000000000000000000000000000000000";


#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn call_get_licenses() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let block_heights = 0..1024u64;
    let response = client.contract_query::<Range<u64>, 0>(LICENSE_CONTRACT, "get_licenses", &block_heights).await?;

    let response_data = check_archived_root::<Vec<(u64,Vec<u8>)>>(response.as_slice())
        .map_err(|_| {
            InvalidQueryResponse(Box::from("rkyv deserialization error"))
        })?;
    let r: Vec<(u64,Vec<u8>)> = response_data
        .deserialize(&mut Infallible)
        .expect("Infallible");

    println!("response={:?}", r);
    Ok(())
}
