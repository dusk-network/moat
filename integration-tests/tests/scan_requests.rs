// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{Error, RequestScanner};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
async fn scan_requests_in_lasts_blocks() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    const SCAN_N_LAST_BLOCKS: u32 = 10000;

    let requests =
        RequestScanner::scan_last_blocks(SCAN_N_LAST_BLOCKS, &cfg).await?;

    println!(
        "there were {} requests found in last n={} blocks",
        requests.len(),
        SCAN_N_LAST_BLOCKS
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
async fn scan_requests_in_block_range() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    const HEIGHT_BEG: u64 = 317042;
    const HEIGHT_END: u64 = 317048;

    let (requests, _) =
        RequestScanner::scan_block_range(HEIGHT_BEG, HEIGHT_END, &cfg).await?;

    println!(
        "there were {} requests found in block range from {} to {}",
        requests.len(),
        HEIGHT_BEG,
        HEIGHT_END
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
// #[cfg_attr(not(feature = "expensive_tests"), ignore)]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
async fn scan_all_requests() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    let mut height = 0;
    loop {
        let height_end = height + 10000;
        let (requests, top) =
            RequestScanner::scan_block_range(height, height_end, &cfg).await?;

        println!(
            "{} requests in range ({},{}) top={}",
            requests.len(),
            height,
            height_end,
            top
        );

        if top <= height_end {
            break;
        }

        height = height_end;
    }

    Ok(())
}
