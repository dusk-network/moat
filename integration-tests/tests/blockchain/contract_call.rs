// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::{RuskHttpClient, WalletPath};
use phoenix_core::transaction::ModuleId;
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use zk_citadel_moat::wallet_accessor::BlockchainAccessConfig;
use zk_citadel_moat::wallet_accessor::Password::PwdHash;
use zk_citadel_moat::{
    Error, JsonLoader, PayloadSender, RequestJson, TxAwaiter,
};

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

pub const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x03;
    bytes
};
// note that any method can be used here as long as it does not require
// arguments
pub const CONTRACT_METHOD_NAME: &str = "request_license";

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn contract_call_with_payload() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let request_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/request/test_request.json"
    );

    let request_json: RequestJson = RequestJson::from_file(request_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let tx_id = PayloadSender::execute_contract_method(
        // any payload will do as long as the called method does not require
        // arguments
        request_json.provider_psk,
        &blockchain_config,
        &wallet_path,
        &PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
        LICENSE_CONTRACT_ID,
        CONTRACT_METHOD_NAME,
    )
    .await?;
    let client = RuskHttpClient::new(blockchain_config.rusk_address.clone());
    TxAwaiter::wait_for(&client, tx_id).await?;

    Ok(())
}
