// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Test for the user citadel journey
//! The following needs to happen:
//!   - first we need to pretend that we are a license provider so that we can
//!     call issue_license and have at least one license available
//!   - we call get_licenses to obtain a license and its position in the merkle
//!     tree
//!   - we call get_merkle_opening to obtain a relevant merkle opening
//!   - we compute the proof and send it via use_license
//!   - at this point we need to pretend that we are a service provider so that
//!     we can call get_session and be sure that we have a session
//!   - at the end we can make sure that the license is used, so a subsequent
//!     try to use the license is not successful (in other words, there is a
//!     nullifier (or session id) in a collection which stops us from double
//!     usage of the license) TBD - this last scenario needs to be cleared with
//!     Xavi still

use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::{CitadelInquirer, Error, JsonLoader, RequestCreator, RequestJson};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::PwdHash;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

async fn issue_license(
    reference_lp: &ReferenceLP,
    blockchain_config: BlockchainAccessConfig,
    wallet_path: WalletPath,
    request_path: impl AsRef<str>,
) -> Result<(), Error> {
    let request_json: RequestJson =
        RequestJson::from_file(request_path.as_ref())?;

    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;

    let license_issuer = LicenseIssuer::new(
        blockchain_config,
        wallet_path,
        PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    );

    license_issuer
        .issue_license(rng, &request, &reference_lp.ssk_lp)
        .await
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn user_round_trip() -> Result<(), Error> {
    // initialize
    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp2.json");

    let reference_lp = ReferenceLP::init(&lp_config_path)?;

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let client = RuskHttpClient::new(blockchain_config.rusk_address.clone());

    // call issue license, wait for tx to confirm

    issue_license(&reference_lp, blockchain_config, wallet_path, request_path).await?;

    // call get_licenses, obtain license and pos

    let block_heights = 0..10000u64; // todo: obtain height

    let licenses = CitadelInquirer::get_licenses(&client, block_heights).await?;
    assert!(!licenses.is_empty());
    let license = licenses[0].1.clone();
    let pos = licenses[0].0.clone();
    println!("license obtained at pos={}", pos);

    // call get_merkle_opening, obtain opening

    let opening = CitadelInquirer::get_merkle_opening(&client, pos).await?;
    assert!(opening.is_some());
    println!("opening obtained={:?}", opening);

    // compute proof, call use_license, wait for tx to confirm
    // call get_session
    Ok(())
}
