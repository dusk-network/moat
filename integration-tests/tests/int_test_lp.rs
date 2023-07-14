// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use license_provider::ReferenceLP;
use moat_core::Error;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn lp_scan() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp.json");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp = ReferenceLP::init(&lp_config_path)?;

    reference_lp.scan(&blockchain_config).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn lp_scan_last_blocks() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp2.json");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp = ReferenceLP::init(&lp_config_path)?;

    let (_total, _owned) = reference_lp
        .scan_last_blocks(10000, &blockchain_config)
        .await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn lp_scan_2_lps() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let lp1_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp.json");
    let lp2_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp2.json");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp1 = ReferenceLP::init(&lp1_config_path)?;
    let mut reference_lp2 = ReferenceLP::init(&lp2_config_path)?;
    let (_, _lp1_count) = reference_lp1.scan(&blockchain_config).await?;
    let (_, _lp2_count) = reference_lp2.scan(&blockchain_config).await?;
    Ok(())
}