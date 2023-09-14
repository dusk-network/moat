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
use moat_core::{CitadelInquirer, Error, JsonLoader, LicenseCircuit, PayloadSender, RequestCreator, RequestJson, TxAwaiter};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use dusk_jubjub::BlsScalar;
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use toml_base_config::BaseConfig;
use zk_citadel::license::Request;
use zk_citadel::utils::CitadelUtils;
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::PwdHash;
use dusk_plonk::prelude::*;
use rkyv::{Archive, Deserialize, Serialize};
use bytecheck::CheckBytes;
use dusk_bytes::DeserializableSlice;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

// todo: proper location for these constants
const DEPTH: usize = 17; // depth of the Merkle tree
const ARITY: usize = 4; // arity of the Merkle tree
static LABEL: &[u8] = b"dusk-network";
const CAPACITY: usize = 17; // capacity required for the setup



/// Use License Argument.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct UseLicenseArg {
    pub proof: Proof,
    pub public_inputs: Vec<BlsScalar>,
}

async fn issue_license(
    reference_lp: &ReferenceLP,
    blockchain_config: BlockchainAccessConfig,
    wallet_path: WalletPath,
    request: &Request,
    rng: &mut StdRng,
) -> Result<(), Error> {
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

async fn use_license(
    client: &RuskHttpClient,
    blockchain_config: &BlockchainAccessConfig,
    wallet_path: WalletPath,
    reference_lp: &ReferenceLP,
    ssk_user: SecretSpendKey,
    psk_user: PublicSpendKey,
    prover: &Prover,
    verifier: &Verifier,
    rng: &mut StdRng,
) -> Result<BlsScalar, Error> {
    let (cpp, sc) =
        CitadelUtils::compute_citadel_parameters::<StdRng, DEPTH, ARITY>(
            rng, ssk_user, psk_user, reference_lp.ssk_lp, reference_lp.psk_lp,
        );
    let circuit = LicenseCircuit::new(&cpp, &sc);

    println!("starting calculating proof");
    let (proof, public_inputs) =
        prover.prove(rng, &circuit).expect("Proving should succeed");
    println!("calculating proof done");

    verifier
        .verify(&proof, &public_inputs)
        .expect("Verifying the circuit should succeed");

    let use_license_arg = UseLicenseArg {
        proof,
        public_inputs,
    };

    let tx_id = PayloadSender::send_use_license(
        use_license_arg,
        &blockchain_config,
        &wallet_path,
        &PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    ).await?;
    TxAwaiter::wait_for(&client, tx_id).await?;
    Ok(tx_id)
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn user_round_trip() -> Result<(), Error> {
    // initialize
    let rng = &mut StdRng::seed_from_u64(0xcafe);

    println!("performing setup");
    let pp = PublicParameters::setup(1 << CAPACITY, rng).unwrap();

    println!("compiling circuit");
    let (prover, verifier) = Compiler::compile::<LicenseCircuit>(&pp, LABEL)
        .expect("Compiling circuit should succeed");
    println!("compiling circuit done");

    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp2.json");

    let reference_lp = ReferenceLP::init(&lp_config_path)?;

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;
    let blockchain_config_clone =
        BlockchainAccessConfig::load_path(blockchain_config_path)?; // todo: eliminate this clone

    let request_json: RequestJson =
        RequestJson::from_file(request_path)?;

    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk.clone(),
        request_json.provider_psk,
        rng,
    )?;

    let ssk_user_bytes = hex::decode(request_json.user_ssk)?;
    let ssk_user = SecretSpendKey::from_slice(ssk_user_bytes.as_slice())?;
    let psk_user = ssk_user.public_spend_key();

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let client = RuskHttpClient::new(blockchain_config.rusk_address.clone());

    // call issue license, wait for tx to confirm

    issue_license(&reference_lp, blockchain_config, wallet_path.clone(), &request, rng).await?; // todo: eliminate clones

    // call get_licenses, obtain license and pos

    let block_heights = 0..10000u64; // todo: obtain height

    let licenses = CitadelInquirer::get_licenses(&client, block_heights).await?;
    assert!(!licenses.is_empty());
    let _license = licenses[0].1.clone(); //todo: explain - why is license not used?
    let pos = licenses[0].0.clone();
    println!("license obtained at pos={}", pos);

    // call get_merkle_opening, obtain opening

    let opening = CitadelInquirer::get_merkle_opening(&client, pos).await?;
    assert!(opening.is_some());
    println!("opening obtained");

    // compute proof, call use_license, wait for tx to confirm
    use_license(&client, &blockchain_config_clone, wallet_path, &reference_lp, ssk_user, psk_user, &prover, &verifier, rng).await?;

    // call get_session
    Ok(())
}
