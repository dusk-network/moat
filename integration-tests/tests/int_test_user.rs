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
//!     usage of the license)

use bytecheck::CheckBytes;
use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_jubjub::BlsScalar;
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_plonk::prelude::*;
use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::Error::InvalidQueryResponse;
use moat_core::{
    BcInquirer, CitadelInquirer, Error, JsonLoader, LicenseCircuit,
    LicenseSessionId, PayloadSender, RequestCreator, RequestJson, TxAwaiter,
    ARITY, DEPTH,
};
use poseidon_merkle::Opening;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rkyv::{check_archived_root, Archive, Deserialize, Infallible, Serialize};
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use tracing::{info, Level};
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::PwdHash;
use zk_citadel::license::{
    CitadelProverParameters, License, Request, SessionCookie,
};

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

static LABEL: &[u8] = b"dusk-network";
const CAPACITY: usize = 17; // capacity required for the setup

/// Use License Argument.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct UseLicenseArg {
    pub proof: Proof,
    pub public_inputs: Vec<BlsScalar>,
}

fn compute_citadel_parameters(
    rng: &mut StdRng,
    ssk: SecretSpendKey,
    psk_lp: PublicSpendKey,
    lic: &License,
    merkle_proof: Opening<(), DEPTH, ARITY>,
) -> (CitadelProverParameters<DEPTH, ARITY>, SessionCookie) {
    const CHALLENGE: u64 = 20221127u64;
    let c = JubJubScalar::from(CHALLENGE);
    let (cpp, sc) = CitadelProverParameters::compute_parameters(
        &ssk,
        &lic,
        &psk_lp,
        &psk_lp,
        &c,
        rng,
        merkle_proof,
    );
    (cpp, sc)
}

/// Calls license contract's issue license method.
/// Awaits for confirmation of the contract-calling transaction.
async fn issue_license(
    reference_lp: &ReferenceLP,
    blockchain_config: &BlockchainAccessConfig,
    wallet_path: &WalletPath,
    request: &Request,
    rng: &mut StdRng,
) -> Result<BlsScalar, Error> {
    let license_issuer = LicenseIssuer::new(
        blockchain_config.clone(),
        wallet_path.clone(),
        PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    );

    license_issuer
        .issue_license(rng, &request, &reference_lp.ssk_lp)
        .await
}

/// Calculates and verified proof, sends proof along with public parameters
/// as arguments to the license contract's use_license method.
/// Awaits for confirmation of the contract-calling transaction.
async fn use_license(
    client: &RuskHttpClient,
    blockchain_config: &BlockchainAccessConfig,
    wallet_path: &WalletPath,
    reference_lp: &ReferenceLP,
    ssk_user: SecretSpendKey,
    prover: &Prover,
    verifier: &Verifier,
    license: &License,
    opening: Opening<(), DEPTH, ARITY>,
    rng: &mut StdRng,
) -> Result<BlsScalar, Error> {
    let (cpp, sc) = compute_citadel_parameters(
        rng,
        ssk_user,
        reference_lp.psk_lp,
        license,
        opening,
    );
    let circuit = LicenseCircuit::new(&cpp, &sc);

    info!("calculating proof");
    let (proof, public_inputs) =
        prover.prove(rng, &circuit).expect("Proving should succeed");

    assert!(!public_inputs.is_empty());
    let session_id = public_inputs[0];

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
    )
    .await?;
    TxAwaiter::wait_for(&client, tx_id).await?;
    Ok(session_id)
}

/// Deserializes license, panics if deserialization fails.
fn deserialise_license(v: &Vec<u8>) -> License {
    let response_data = check_archived_root::<License>(v.as_slice())
        .map_err(|_| {
            InvalidQueryResponse(Box::from("rkyv deserialization error"))
        })
        .expect("License should deserialize correctly");
    let license: License = response_data
        .deserialize(&mut Infallible)
        .expect("Infallible");
    license
}

/// Displays license contract current state summary.
async fn show_state(
    client: &RuskHttpClient,
    s: impl AsRef<str>,
) -> Result<(), Error> {
    let (num_licenses, tree_len, num_sessions) =
        CitadelInquirer::get_info(&client).await?;
    info!(
        "contract state {} - licenses: {} tree length: {} sessions: {}",
        s.as_ref(),
        num_licenses,
        tree_len,
        num_sessions
    );
    Ok(())
}

/// Finds owned license in a collection of licenses.
/// It searches in a reverse order to return a newest license.
fn find_owned_license(
    ssk_user: SecretSpendKey,
    licenses: Vec<(u64, Vec<u8>)>,
) -> Option<(u64, License)> {
    for (pos, license) in licenses.iter().rev() {
        let license = deserialise_license(&license);
        if ssk_user.view_key().owns(&license.lsa) {
            return Some((pos.clone(), license));
        }
    }
    None
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "exp_tests"), ignore)]
async fn user_round_trip() -> Result<(), Error> {
    const BLOCK_RANGE: u64 = 10000;
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr);
    tracing::subscriber::set_global_default(subscriber.finish())
        .expect("Setting tracing default should work");

    // initialize
    // NOTE: it is important that the seed is the same as in the recovery
    // PUB_PARAMS initialization code
    let rng = &mut StdRng::seed_from_u64(0xbeef);

    info!("performing setup");
    let pp = PublicParameters::setup(1 << CAPACITY, rng).unwrap();

    info!("compiling circuit");
    let (prover, verifier) = Compiler::compile::<LicenseCircuit>(&pp, LABEL)
        .expect("Compiling circuit should succeed");

    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");
    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/config.toml");

    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/lp2.json");

    let reference_lp = ReferenceLP::init(&lp_config_path)?;

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let request_json: RequestJson = RequestJson::from_file(request_path)?;

    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk.clone(),
        request_json.provider_psk,
        rng,
    )?;

    let ssk_user_bytes = hex::decode(request_json.user_ssk)?;
    let ssk_user = SecretSpendKey::from_slice(ssk_user_bytes.as_slice())?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    let client = RuskHttpClient::new(blockchain_config.rusk_address.clone());

    // as a LP, call issue license, wait for tx to confirm

    show_state(&client, "before issue_license").await?;
    info!("calling issue_license (as an LP)");
    let issue_license_txid = issue_license(
        &reference_lp,
        &blockchain_config,
        &wallet_path,
        &request,
        rng,
    )
    .await?;
    show_state(&client, "after issue_license").await?;
    TxAwaiter::wait_for(&client, issue_license_txid).await?;
    let end_height = BcInquirer::block_height(&client).await?;
    info!("end_height={}", end_height);

    // as a User, call get_licenses, obtain license and pos

    let start_height = if end_height > BLOCK_RANGE {
        end_height - BLOCK_RANGE
    } else {
        0u64
    };
    let block_heights = start_height..(end_height.clone() + 1);

    info!(
        "calling get_licenses with range {:?} (as a user)",
        block_heights
    );
    let licenses =
        CitadelInquirer::get_licenses(&client, block_heights).await?;
    assert!(!licenses.is_empty());

    let (pos, license) =
        find_owned_license(ssk_user, licenses).expect("owned license found");

    // as a User, call get_merkle_opening, obtain opening

    info!("calling get_merkle_opening (as a user)");
    let opening = CitadelInquirer::get_merkle_opening(&client, pos).await?;
    assert!(opening.is_some());

    // as a User, compute proof, call use_license, wait for tx to confirm

    show_state(&client, "before use_license").await?;
    info!("calling use_license (as a user)");
    let session_id = use_license(
        &client,
        &blockchain_config,
        &wallet_path,
        &reference_lp,
        ssk_user,
        &prover,
        &verifier,
        &license,
        opening.unwrap(),
        rng,
    )
    .await?;
    show_state(&client, "after use_license").await?;
    let session_id = LicenseSessionId { id: session_id };

    // as an SP, call get_session

    info!("calling get_session (as an SP)");
    let session = CitadelInquirer::get_session(&client, session_id).await?;
    assert!(session.is_some());
    let session = session.unwrap();
    info!(
        "obtained session {}",
        hex::encode(session.public_inputs[0].to_bytes())
    );

    // if we try to call use_license again, it will be rejected
    // (currently, it panics giving Trap(UnreachableCodeReached))
    Ok(())
}
