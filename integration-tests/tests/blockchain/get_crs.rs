// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::RuskHttpClient;
use zk_citadel_moat::wallet_accessor::BlockchainAccessConfig;
use zk_citadel_moat::{CrsGetter, Error};
use toml_base_config::BaseConfig;
use tracing::trace;

const MIN_CRS_SIZE: usize = 10 * 1024 * 1024;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn get_crs() -> Result<(), Error> {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let client = RuskHttpClient::new(cfg.rusk_address);

    let crs = CrsGetter::get_crs(&client).await?;

    assert!(crs.len() >= MIN_CRS_SIZE);

    trace!("crs={}...", hex::encode(&crs[0..64]));
    trace!("crs length={}", crs.len());

    Ok(())
}
