// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::JubJubScalar;
use dusk_wallet::WalletPath;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use zk_citadel_moat::license_provider::{LicenseIssuer, ReferenceLP};
use zk_citadel_moat::wallet_accessor::BlockchainAccessConfig;
use zk_citadel_moat::wallet_accessor::Password::PwdHash;
use zk_citadel_moat::{Error, JsonLoader, RequestCreator, RequestJson};

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn issue_license() -> Result<(), Error> {
    let request_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/request/test_request.json"
    );
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

    let lp_config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/test_keys/test_keys_lp_2.json"
    );

    let reference_lp = ReferenceLP::create(&lp_config_path)?;

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

    let license_issuer = LicenseIssuer::new(
        blockchain_config,
        wallet_path,
        PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    );

    const ATTRIBUTE_DATA_EXAMPLE: u64 = 1234;

    license_issuer
        .issue_license(
            rng,
            &request,
            &reference_lp.ssk_lp,
            &JubJubScalar::from(ATTRIBUTE_DATA_EXAMPLE),
        )
        .await?;

    Ok(())
}
