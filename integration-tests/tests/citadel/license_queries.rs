// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bls12_381::BlsScalar;
use dusk_wallet::RuskHttpClient;
use moat::wallet_accessor::BlockchainAccessConfig;
use moat::{CitadelInquirer, Error, LicenseSessionId, StreamAux};
use toml_base_config::BaseConfig;
use tracing::trace;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn call_get_licenses() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let block_heights = 0..5000u64;

    let stream = CitadelInquirer::get_licenses(&client, block_heights).await?;

    const ITEM_LEN: usize = CitadelInquirer::GET_LICENSES_ITEM_LEN;
    let response = StreamAux::collect_all::<(u64, Vec<u8>), ITEM_LEN>(stream)?;
    trace!("response={:?}", response);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn call_get_merkle_opening() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let pos = 0u64;

    let response = CitadelInquirer::get_merkle_opening(&client, pos).await?;
    trace!("response={:?}", response);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn call_get_session() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let response = CitadelInquirer::get_session(
        &client,
        LicenseSessionId {
            id: BlsScalar::one(),
        },
    )
    .await?;
    trace!("response={:?}", response);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn call_get_info() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let config = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(config.rusk_address);

    let response = CitadelInquirer::get_info(&client).await?;
    trace!("response={:?}", response);
    Ok(())
}
