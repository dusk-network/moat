// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::WalletPath;
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::{Error, JsonLoader, RequestCreator, RequestJson};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::PwdHash;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787";
const GAS_LIMIT: u64 = 500_000_000_000-1;
const GAS_PRICE: u64 = 1;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn issue_license() -> Result<(), Error> {
    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp2.json");

    let reference_lp = ReferenceLP::init(&lp_config_path)?;

    let request_json: RequestJson = RequestJson::from_file(request_path)?;

    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    println!("before LicenseIssuer::new");

    let license_issuer = LicenseIssuer::new(
        blockchain_config,
        wallet_path,
        PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    );

    println!("before issue_license");

    // todo: pos needs to be determined in contract, and removed from the
    // interface
    let license_pos = 1u64;
    license_issuer
        .issue_license(rng, &request, &reference_lp.ssk_lp, license_pos)
        .await?;

    Ok(())
}
