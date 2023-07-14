// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gql_client::Client;
use moat_core::{Error, PayloadRetriever, RequestScanner};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn retrieve_payload() -> Result<(), Error> {
    const TXID: &str =
        "5f486c6f4edc9321e15a83993aa68463e733fc482acbde979881450c83c92a0e";

    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let config = BlockchainAccessConfig::load_path(config_path)?;
    let client = Client::new(config.graphql_address.clone());

    let request: Request =
        PayloadRetriever::retrieve_payload(TXID, &client).await?;
    println!("request={:?}", request);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
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

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn scan_requests_in_lasts_blocks() -> Result<(), Error> {
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let cfg = BlockchainAccessConfig::load_path(config_path)?;

    const LAST_BLOCKS: usize = 10000;

    let requests = RequestScanner::scan_last_blocks(LAST_BLOCKS, &cfg).await?;
    println!(
        "there were {} requests found in last n={} blocks",
        requests.len(),
        LAST_BLOCKS
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
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