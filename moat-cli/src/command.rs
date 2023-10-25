// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::SeedableRng;
use dusk_wallet::{RuskHttpClient, WalletPath};
use moat_core::{Error, RequestCreator, RequestJson, RequestSender, TxAwaiter};
use rand::rngs::StdRng;
use wallet_accessor::{BlockchainAccessConfig, Password};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Submit request
    SubmitRequest { dummy: bool },
    /// List requests
    ListRequests { dummy: bool },
}

impl Command {
    pub async fn run(
        self,
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
        request_json: Option<RequestJson>,
    ) -> Result<(), Error> {
        match self {
            Command::SubmitRequest { dummy: true } => {
                let request_json =
                    request_json.expect("request should be provided"); // todo
                                                                       // todo - this request creation belongs somewhere else because
                                                                       // we might
                                                                       // also want to create request on the fly, from data provided by
                                                                       // user interactively
                let rng = &mut StdRng::seed_from_u64(0xcafe);
                let request = RequestCreator::create_from_hex_args(
                    request_json.user_ssk,
                    request_json.provider_psk,
                    rng,
                )?;
                println!("submitting request");
                let tx_id = RequestSender::send_request(
                    request,
                    blockchain_access_config,
                    wallet_path,
                    psw,
                    gas_limit,
                    gas_price,
                )
                .await?;
                println!(
                    "tx {} submitted, waiting for confirmation",
                    hex::encode(tx_id.to_bytes())
                );
                let client = RuskHttpClient::new(
                    blockchain_access_config.rusk_address.clone(),
                );
                TxAwaiter::wait_for(&client, tx_id).await?;
                println!("tx {} confirmed", hex::encode(tx_id.to_bytes()));
            }
            Command::ListRequests { dummy: true } => (),
            _ => (),
        }
        Ok(())
    }
}
