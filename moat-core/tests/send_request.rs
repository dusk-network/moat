// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::WalletPath;
use moat_core::{Error, RequestJson, RequestSender};
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PASSWORD: &str = "hyundai23!";
const GAS_LIMIT: u64 = 500_000_000;
const GAS_PRICE: u64 = 1;

#[tokio::test(flavor = "multi_thread")]
async fn send_request() -> Result<(), Error> {
    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request.json");
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let request_json = RequestJson::from_file(request_path)?;

    let request = request_json.to_request();

    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    RequestSender::send(
        request,
        &blockchain_access_config,
        wallet_path,
        PASSWORD.to_string(),
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;

    Ok(())
}
