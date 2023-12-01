// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use zk_citadel_moat::license_provider::ReferenceLP;
use zk_citadel_moat::wallet_accessor::BlockchainAccessConfig;
use zk_citadel_moat::Error;
use toml_base_config::BaseConfig;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn lp_scan() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");
    let lp_config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/test_keys/test_keys_lp.json"
    );

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp = ReferenceLP::create(&lp_config_path)?;

    reference_lp.scan(&blockchain_config).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn lp_scan_last_blocks() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");
    let lp_config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/test_keys/test_keys_lp_2.json"
    );

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp = ReferenceLP::create(&lp_config_path)?;

    let (_total, _owned) = reference_lp
        .scan_last_blocks(10000, &blockchain_config)
        .await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn lp_scan_2_lps() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");
    let lp1_config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/test_keys/test_keys_lp.json"
    );
    let lp2_config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/test_keys/test_keys_lp_2.json"
    );

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp1 = ReferenceLP::create(&lp1_config_path)?;
    let mut reference_lp2 = ReferenceLP::create(&lp2_config_path)?;
    let (_, _lp1_count) = reference_lp1.scan(&blockchain_config).await?;
    let (_, _lp2_count) = reference_lp2.scan(&blockchain_config).await?;
    Ok(())
}
