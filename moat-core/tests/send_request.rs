// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::{Path, PathBuf};
use dusk_wallet::WalletPath;
use moat_core::{Error, RequestJson, RequestSender};
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};
use toml_base_config::BaseConfig;

#[tokio::test(flavor = "multi_thread")]
async fn send_request() -> Result<(), Error> {

    let request_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request.json");
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config.toml");

    let request_json = RequestJson::from_file(request_path)?;

    let request = request_json.to_request();

    let blockchain_access_config = BlockchainAccessConfig::load_path(config_path)?;

    // todo: missing proper path to wallet passing mechanism here
    let wallet_path =
        WalletPath::from(PathBuf::from("/Users/miloszm/.dusk/rusk-wallet").as_path().join("wallet.dat"));
    // todo: missing proper password passing mechanism here
    let password = String::from("hyundai23!");

    println!("11={:?}", request);
    println!("22={:?}", blockchain_access_config);

    RequestSender::send(request, &blockchain_access_config, wallet_path, password).await?;

    Ok(())
}
