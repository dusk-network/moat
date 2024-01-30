// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::RuskHttpClient;
use phoenix_core::transaction::ModuleId;
use zk_citadel_moat::{
    Error, JsonLoader, PayloadSender, RequestJson, TxAwaiter,
};

use zk_citadel_moat::api::MoatContext;

const WALLET_PATH: &str =
    concat!(env!("HOME"), "/.dusk/rusk-wallet/wallet.dat");
const WALLET_PASS: &str = "password";
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

    let request_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/request/test_request.json"
    );

    let request_json: RequestJson = RequestJson::from_file(request_path)?;

    let moat_context = MoatContext::create(
        blockchain_config_path,
        WALLET_PATH,
        WALLET_PASS,
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;

    let tx_id = PayloadSender::execute_contract_method(
        // any payload will do as long as the called method does not require
        // arguments
        request_json.provider_psk,
        &moat_context,
        LICENSE_CONTRACT_ID,
        CONTRACT_METHOD_NAME,
    )
    .await?;
    let client = RuskHttpClient::new(
        moat_context.blockchain_access_config.rusk_address.clone(),
    );
    TxAwaiter::wait_for(&client, tx_id).await?;

    Ok(())
}
