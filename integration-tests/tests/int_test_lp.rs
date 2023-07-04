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
#[cfg_attr(not(feature = "expensive_tests"), ignore)]
async fn lp_run() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp.json");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let mut reference_lp = ReferenceLP::init(&lp_config_path)?;

    reference_lp.scan(&blockchain_config).await?;

    assert!(reference_lp.requests_to_process.len() > 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "expensive_tests"), ignore)]
async fn lp_run_2_lps() -> Result<(), Error> {
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
    let _ = reference_lp1.scan(&blockchain_config).await?;
    let total = reference_lp2.scan(&blockchain_config).await?;

    let lp1_count = reference_lp1.requests_to_process.len();
    let lp2_count = reference_lp2.requests_to_process.len();
    assert!( lp1_count > 0);
    assert!( lp2_count > 0);
    assert!( (lp1_count + lp2_count) <= total);

    Ok(())
}
