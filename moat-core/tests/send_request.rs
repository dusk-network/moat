// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::WalletPath;
use moat_core::{
    Error, PayloadRetriever, RequestCreator, RequestJson, RequestSender,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PASSWORD: &str = "password";
const GAS_LIMIT: u64 = 500_000_000;
const GAS_PRICE: u64 = 1;

#[tokio::test(flavor = "multi_thread")]
async fn send_request() -> Result<(), Error> {
    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");
    let config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let request_json = RequestJson::from_file(request_path)?;

    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;
    let request_vec = rkyv::to_bytes::<_, 8192>(&request).unwrap().to_vec();

    let bac = BlockchainAccessConfig::load_path(config_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let tx_id = RequestSender::send(
        request,
        &bac,
        wallet_path,
        PASSWORD.to_string(),
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;

    let tx_id_hex = format!("{:x}", tx_id);
    let retrieved_request =
        get_request_from_blockchain(tx_id_hex, &bac).await?;
    assert_eq!(
        request_vec,
        rkyv::to_bytes::<_, 8192>(&retrieved_request)
            .unwrap()
            .to_vec(),
        "requests not equal"
    );

    Ok(())
}

async fn get_request_from_blockchain<S: AsRef<str>>(
    tx_id: S,
    bac: &BlockchainAccessConfig,
) -> Result<Request, Error> {
    const NUM_RETRIES: i32 = 20;
    for i in 0..NUM_RETRIES {
        let result =
            PayloadRetriever::retrieve_tx_payload(tx_id.as_ref().clone(), &bac)
                .await;
        if result.is_err() && i < (NUM_RETRIES - 1) {
            let _ = sleep(Duration::from_millis(1000));
            continue;
        }
        return result;
    }
    unreachable!()
}
