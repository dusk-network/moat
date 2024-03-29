// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::{RuskHttpClient, WalletPath};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use toml_base_config::BaseConfig;
use tracing::Level;
use zk_citadel::license::Request;
use zk_citadel_moat::wallet_accessor::{
    BlockchainAccessConfig, Password::PwdHash,
};
use zk_citadel_moat::{
    Error, PayloadExtractor, PayloadRetriever, RequestCreator, RequestJson,
    RequestSender, TxInquirer, MAX_REQUEST_SIZE,
};
use zk_citadel_moat::{JsonLoader, TxAwaiter};

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn send_request() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr);
    tracing::subscriber::set_global_default(subscriber.finish())
        .expect("Setting tracing default should work");

    let request_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/request/test_request.json"
    );
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

    let request_json: RequestJson = RequestJson::from_file(request_path)?;

    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;
    let request_vec = rkyv::to_bytes::<_, MAX_REQUEST_SIZE>(&request)
        .expect("Serializing should be infallible")
        .to_vec();

    let config = BlockchainAccessConfig::load_path(config_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let tx_id = RequestSender::send_request(
        request,
        &config,
        &wallet_path,
        &PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;
    let client = RuskHttpClient::new(config.rusk_address);
    TxAwaiter::wait_for(&client, tx_id).await?;

    let tx_id_hex = hex::encode(tx_id.to_bytes());

    let retrieved_request =
        get_request_from_blockchain(&tx_id_hex, &client).await?;
    assert_eq!(
        request_vec,
        rkyv::to_bytes::<_, MAX_REQUEST_SIZE>(&retrieved_request)
            .expect("Serializing should be infallible")
            .to_vec(),
        "requests not equal"
    );

    let retrieved_request =
        get_request_from_blockchain_bulk(&tx_id_hex, &client).await?;
    assert_eq!(
        request_vec,
        rkyv::to_bytes::<_, MAX_REQUEST_SIZE>(&retrieved_request)
            .expect("Serializing should be infallible")
            .to_vec(),
        "requests not equal"
    );

    Ok(())
}

async fn get_request_from_blockchain<S: AsRef<str>>(
    tx_id: S,
    client: &RuskHttpClient,
) -> Result<Request, Error> {
    const NUM_RETRIES: i32 = 30;
    for i in 0..NUM_RETRIES {
        let result =
            PayloadRetriever::retrieve_payload(tx_id.as_ref().clone(), client)
                .await;
        if result.is_err() && i < (NUM_RETRIES - 1) {
            let _ = sleep(Duration::from_millis(1000)).await;
            continue;
        }
        return result;
    }
    unreachable!()
}

async fn get_request_from_blockchain_bulk<S: AsRef<str>>(
    tx_id: S,
    client: &RuskHttpClient,
) -> Result<Request, Error> {
    const LAST_N_BLOCKS: usize = 1000;
    let txs =
        TxInquirer::txs_from_last_n_blocks(&client, LAST_N_BLOCKS).await?;
    for tx in txs.transactions.iter() {
        if tx.id == tx_id.as_ref() {
            return PayloadExtractor::payload_from_tx(&tx);
        }
    }
    unreachable!()
}
