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

use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_pki::SecretSpendKey;
use dusk_plonk::prelude::*;
use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::{
    BcInquirer, CitadelInquirer, CrsGetter, Error, JsonLoader, LicenseCircuit,
    LicenseSessionId, LicenseUser, PayloadRetriever, RequestCreator,
    RequestJson, RequestSender, TxAwaiter,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;
use toml_base_config::BaseConfig;
use tracing::{info, Level};
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::PwdHash;
use zk_citadel::license::Request;

const WALLET_PATH: &str = concat!(env!("HOME"), "/.dusk/rusk-wallet");
const PWD_HASH: &str =
    "9afbce9f2416520733bacb370315d32b6b2c43d6097576df1c1222859d91eecc";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

static LABEL: &[u8] = b"dusk-network";

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

    let (tx_id, _) = license_issuer
        .issue_license(rng, &request, &reference_lp.ssk_lp)
        .await?;
    Ok(tx_id)
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

///
/// test user_round_trip realizes the following scenario:
/// - creates request (User)
/// - based on the request, calls issue_license (LP)
/// - calls get_licenses, obtains license and position (User)
/// - calls get_merkle_opening for a given position, obtains the opening (User)
/// - based on license and opening, computes proof (User)
/// - calls use_license and creates a session_id (User)
/// - use_license verifies the proof, creates the corresponding session (License
///   Contract)
/// - calls get_session based on a session id given to the SP by the User (SP)
///
/// - Note that session_id is created by the User and not returned by
///   use_license. Although use_license, internally, also creates the same
///   session_id, it is not returned by it because contract state changing
///   methods do not have the ability to return values. It is assumed that
///   license_id created by the user and by contract are the same.
/// - Note that after each contract method call the test waits for transaction
///   to confirm.
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
    let mut rng = StdRng::seed_from_u64(0xbeef);

    let blockchain_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "../../config.toml");

    let blockchain_config =
        BlockchainAccessConfig::load_path(blockchain_config_path)?;

    let client = RuskHttpClient::new(blockchain_config.rusk_address.clone());

    info!("obtaining CRS");
    let pp_vec = CrsGetter::get_crs(&client).await?;
    let pp =
        // SAFETY: CRS vector is checked by the hash check when it is received from the node
        unsafe { PublicParameters::from_slice_unchecked(pp_vec.as_slice()) };

    info!("compiling circuit");
    let (prover, verifier) = Compiler::compile::<LicenseCircuit>(&pp, LABEL)
        .expect("Compiling circuit should succeed");

    let request_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/request/request.json");

    let lp_config_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config/test_secret_key_lp_2");

    let reference_lp = ReferenceLP::create(&lp_config_path)?;

    let wallet_path = WalletPath::from(
        PathBuf::from(WALLET_PATH).as_path().join("wallet.dat"),
    );

    // create request
    let request_json: RequestJson = RequestJson::from_file(request_path)?;
    let ssk_user_bytes = hex::decode(request_json.user_ssk.clone())?;
    let ssk_user = SecretSpendKey::from_slice(ssk_user_bytes.as_slice())?;

    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        &mut rng,
    )?;

    // as a User, submit request to blockchain
    info!("submitting request to blockchain (as a User)");
    let tx_id = RequestSender::send_request(
        request,
        &blockchain_config,
        &wallet_path,
        &PwdHash(PWD_HASH.to_string()),
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;
    TxAwaiter::wait_for(&client, tx_id).await?;

    // as a LP, retrieve request from blockchain
    info!("retrieving request from blockchain (as an LP)");
    let tx_id = hex::encode(tx_id.to_bytes());
    let request: Request =
        PayloadRetriever::retrieve_payload(tx_id, &client).await?;

    // as a LP, call issue license, wait for tx to confirm
    show_state(&client, "before issue_license").await?;
    info!("calling issue_license (as an LP)");
    let issue_license_txid = issue_license(
        &reference_lp,
        &blockchain_config,
        &wallet_path,
        &request,
        &mut rng,
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
    let mut licenses_stream =
        CitadelInquirer::get_licenses(&client, block_heights).await?;

    let owned_licenses =
        CitadelInquirer::find_owned_licenses(ssk_user, &mut licenses_stream)?;
    let (pos, license) = owned_licenses.last().expect("owned license found");

    // as a User, call get_merkle_opening, obtain opening
    info!("calling get_merkle_opening (as a user)");
    let opening =
        CitadelInquirer::get_merkle_opening(&client, pos.clone()).await?;
    assert!(opening.is_some());

    // as a User, compute proof, call use_license, wait for tx to confirm
    show_state(&client, "before use_license").await?;
    // for test purposes we make challenge dependent on the number of sessions,
    // so that it is different every time we run the test
    let (_, _, num_sessions) = CitadelInquirer::get_info(&client).await?;
    let challenge = JubJubScalar::from(num_sessions as u64 + 1);
    info!("proving license and calling use_license (as a user)");
    let (tx_id, session_cookie) = LicenseUser::prove_and_use_license(
        &blockchain_config,
        &wallet_path,
        &PwdHash(PWD_HASH.to_string()),
        &ssk_user,
        &reference_lp.psk_lp,
        &prover,
        &verifier,
        &license,
        opening.unwrap(),
        &mut rng,
        &challenge,
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;
    TxAwaiter::wait_for(&client, tx_id).await?;

    show_state(&client, "after use_license").await?;
    let session_id = LicenseSessionId {
        id: session_cookie.session_id,
    };

    // as an SP, call get_session
    info!("calling get_session (as an SP)");
    let session = CitadelInquirer::get_session(&client, session_id).await?;
    assert!(session.is_some());
    let session = session.unwrap();
    info!(
        "obtained session {}",
        hex::encode(session.public_inputs[0].to_bytes())
    );

    // if we try to call use_license with the same license again, it will be
    // rejected
    Ok(())
}
