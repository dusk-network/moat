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
use dusk_plonk::prelude::*;
use dusk_wallet::RuskHttpClient;
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use tracing::{info, Level};
use zk_citadel::license::Request;
use zk_citadel_moat::license_provider::LicenseIssuer;
use zk_citadel_moat::{
    BcInquirer, CitadelInquirer, Error, LicenseSessionId, LicenseUser,
    PayloadRetriever, TxAwaiter,
};

use zk_citadel_moat::api::{MoatContext, MoatCore};

const WALLET_PATH: &str =
    concat!(env!("HOME"), "/.dusk/rusk-wallet/wallet.dat");
const WALLET_PASS: &str = "password";
const GAS_LIMIT: u64 = 5_000_000_000;
const GAS_PRICE: u64 = 1;

/// Calls license contract's issue license method.
/// Awaits for confirmation of the contract-calling transaction.
async fn issue_license(
    moat_context: &MoatContext,
    request: &Request,
    rng: &mut StdRng,
) -> Result<BlsScalar, Error> {
    const ATTRIBUTE_DATA_EXAMPLE: u64 = 1234;

    let (tx_id, _) = LicenseIssuer::issue_license(
        rng,
        &request,
        &JubJubScalar::from(ATTRIBUTE_DATA_EXAMPLE),
        &moat_context,
    )
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
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

    let moat_context = MoatContext::create(
        blockchain_config_path,
        WALLET_PATH,
        WALLET_PASS,
        GAS_LIMIT,
        GAS_PRICE,
    )
    .await?;

    let client = RuskHttpClient::new(
        moat_context.blockchain_access_config.rusk_address.clone(),
    );
    let (psk_lp, _ssk_lp) = MoatCore::get_wallet_keypair(&moat_context)?;

    // as a User, submit request to blockchain
    info!("submitting request to blockchain (as a User)");
    let (_request_hash, request_tx_id) =
        MoatCore::request_license(&psk_lp, &moat_context, &mut OsRng).await?;
    TxAwaiter::wait_for(&client, request_tx_id).await?;

    // as a LP, retrieve request from blockchain
    info!("retrieving request from blockchain (as an LP)");
    let tx_id = hex::encode(request_tx_id.to_bytes());
    let request: Request =
        PayloadRetriever::retrieve_payload(tx_id, &client).await?;

    // as a LP, call issue license, wait for tx to confirm
    show_state(&client, "before issue_license").await?;
    info!("calling issue_license (as an LP)");
    let issue_license_txid =
        issue_license(&moat_context, &request, &mut rng).await?;
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

    let owned_licenses = CitadelInquirer::find_owned_licenses(
        &moat_context,
        &mut licenses_stream,
    )?;
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
        &moat_context,
        &psk_lp,
        &psk_lp,
        &license,
        opening.expect("opening should be present"),
        &mut OsRng,
        &challenge,
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
    let session = session.expect("session should be present");
    info!(
        "obtained session {}",
        hex::encode(session.public_inputs[0].to_bytes())
    );

    // if we try to call use_license with the same license again, it will be
    // rejected
    Ok(())
}
