// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use license_provider::ReferenceLP;
use moat_core::{Error, JsonLoader, RequestScanner, Transactions};
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "lp"), ignore)]
async fn lp_run() -> Result<(), Error> {
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

#[test]
fn lp_filter_requests() -> Result<(), Error>  {
    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp.json");
    let reference_lp = ReferenceLP::init(&lp_config_path)?;

    let txs_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/transactions.json");

    let txs = Transactions::from_file(txs_path)
        .expect("transactions file should load correctly");

    let requests = RequestScanner::scan_transactions(txs);

    let relevant_requests = reference_lp.relevant_requests(&requests)?;

    println!("filtered {} relevant requests out of total {}", relevant_requests.len(), requests.len());

    assert_eq!(requests.len(), 11);
    assert_eq!(relevant_requests.len(), 9);

    Ok(())
}
