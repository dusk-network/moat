// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::JubJubScalar;
use dusk_wallet::RuskHttpClient;
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use zk_citadel::license::Request;
use zk_citadel_moat::{Error, PayloadRetriever};

use zk_citadel_moat::api::{MoatContext, MoatCore};

const WALLET_PATH: &str =
    concat!(env!("HOME"), "/.dusk/rusk-wallet/wallet.dat");
const WALLET_PASS: &str = "password";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn issue_license() -> Result<(), Error> {
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

    let moat_context = MoatContext::create(
        blockchain_config_path,
        WALLET_PATH,
        WALLET_PASS,
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;

    let (psk_lp, _ssk_lp) = MoatCore::get_wallet_keypair(&moat_context)?;

    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let (_request_hash, request_tx_id) =
        MoatCore::request_license(&psk_lp, &moat_context, &mut OsRng).await?;

    let attr_data = JubJubScalar::from(1234u64);

    let tx_id = hex::encode(request_tx_id.to_bytes());
    let client = RuskHttpClient::new(
        moat_context.blockchain_access_config.rusk_address.clone(),
    );
    let request: Request =
        PayloadRetriever::retrieve_payload(tx_id, &client).await?;

    MoatCore::issue_license(&request, &moat_context, &attr_data, rng).await?;

    Ok(())
}
