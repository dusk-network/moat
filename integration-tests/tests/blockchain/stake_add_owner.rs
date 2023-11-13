// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::{RuskHttpClient, WalletPath};
use moat_core::{Error, JsonLoader, PayloadSender, RequestJson, TxAwaiter};
use phoenix_core::transaction::ModuleId;
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::PwdHash;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "9afbce9f2416520733bacb370315d32b6b2c43d6097576df1c1222859d91eecc";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

pub const STAKE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x02;
    bytes
};
pub const ADD_OWNER_METHOD_NAME: &str = "add_owner";

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn stake_add_owner() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "../../config.toml");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");

    let request_json: RequestJson = RequestJson::from_file(request_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let tx_id = PayloadSender::execute_contract_method(
        request_json.provider_psk,
        &blockchain_config,
        &wallet_path,
        &PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
        STAKE_CONTRACT_ID,
        ADD_OWNER_METHOD_NAME,
    )
    .await?;
    let client = RuskHttpClient::new(blockchain_config.rusk_address.clone());
    TxAwaiter::wait_for(&client, tx_id).await?;

    Ok(())
}
