// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::JubJubScalar;
use rand::rngs::OsRng;

use moat::api::{Error, MoatContext, MoatCore};

#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Error> {
    // Specify a configuration file path to connect to the Blockchain
    let config_path = "./config.toml";

    // Specify a wallet file path and its encryption password
    let wallet_path = concat!(env!("HOME"), "/.dusk/rusk-wallet/wallet.dat");
    let wallet_password = "password";

    // Specify the gas configuration for the transactions
    let gas_limit = 500000000;
    let gas_price = 1;

    // Build a configuration object with the previously set information
    let moat_context = MoatContext::create(
        config_path,
        wallet_path,
        wallet_password,
        gas_limit,
        gas_price,
    )?;

    // Retrieve the keypair from the installed wallet
    let (psk_user, ssk_user) = MoatCore::get_wallet_keypair(&moat_context)?;

    // Submit a request to the Blockchain
    let psk_lp = psk_user; // we specify the same key just for testing
    let request_hash = MoatCore::request_license(
        &ssk_user,
        &psk_lp,
        &moat_context,
        &mut OsRng,
    )
    .await?;
    println!("Request transacted: {:?}", request_hash);

    // Get owned requests
    let ssk_lp = ssk_user; // we set the same key just for testing
    let requests = MoatCore::get_owned_requests(&ssk_lp, &moat_context).await?;

    // Issue a license
    let attr_data = JubJubScalar::from(
        "1234".parse::<u64>().expect("attribute date is correct"),
    );
    let rng = &mut OsRng;
    let license_hash = MoatCore::issue_license(
        &requests.get(0).expect("A request was owned."),
        &ssk_lp,
        &moat_context,
        &attr_data,
        rng,
    )
    .await?;
    println!("License issued: {:?}", license_hash);

    // Get owned licenses
    let licenses =
        MoatCore::get_owned_licenses(&ssk_user, &moat_context).await?;

    // Use a license
    let psk_sp = psk_lp; // we set the same one than the LP just for testing
    let challenge = JubJubScalar::from(
        "1234".parse::<u64>().expect("challenge value is correct"),
    );
    let session_cookie = MoatCore::use_license(
        &moat_context,
        &psk_lp,
        &psk_sp,
        &ssk_user,
        &challenge,
        &licenses.get(0).expect("A license was owned."),
        rng,
    )
    .await?
    .expect("session cookie has been obtained");
    println!("Session cookie for the used license: {:?}", session_cookie);

    // Verify a session cookie
    if MoatCore::verify_requested_service(
        &moat_context,
        &psk_lp,
        &psk_sp,
        &session_cookie,
    )
    .await?
    {
        println!("Session Cookie was correct, service should be granted.");
    } else {
        println!("Session Cookie was not correct, service must be denied.");
    }
    Ok(())
}
