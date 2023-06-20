// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{Error, RequestScanner};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
async fn scan_requests() -> Result<(), Error> {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    const SCAN_N_LAST_BLOCKS: u32 = 10000;

    let requests =
        RequestScanner::scan_last_blocks(SCAN_N_LAST_BLOCKS, &cfg).await?;

    println!("requests={:?}", requests);
    println!("there were {} requests found", requests.len());

    Ok(())
}
