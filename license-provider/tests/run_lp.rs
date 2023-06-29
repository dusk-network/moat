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
#[cfg_attr(not(feature = "lp"), ignore)]
async fn run_license_provider() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");
    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp.json");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let reference_lp = ReferenceLP::init(&lp_config_path)?;

    reference_lp.run(&blockchain_config).await?;

    Ok(())
}
