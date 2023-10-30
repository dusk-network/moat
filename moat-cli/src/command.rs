// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::SeedableRng;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::ReferenceLP;
use moat_core::{
    Error, RequestCreator, RequestJson, RequestScanner, RequestSender,
    TxAwaiter,
};
use rand::rngs::StdRng;
use std::path::Path;
use wallet_accessor::{BlockchainAccessConfig, Password, WalletAccessor};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Submit request
    SubmitRequest { provider_psk: String },
    /// List requests (User)
    ListRequestsUser { dummy: bool },
    /// List requests (LP)
    ListRequestsLP { dummy: bool },
}

impl Command {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        self,
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        lp_config: &Path,
        gas_limit: u64,
        gas_price: u64,
        request_json: Option<RequestJson>,
    ) -> Result<(), Error> {
        match self {
            Command::SubmitRequest { provider_psk } => {
                let request_json =
                    request_json.expect("request should be provided"); // todo
                                                                       // todo - this request creation belongs somewhere else because
                                                                       // we might
                                                                       // also want to create request on the fly, from data provided by
                                                                       // user interactively
                let rng = &mut StdRng::seed_from_u64(0xcafe);
                println!("obtained provider psk={}", provider_psk);
                let provider_psk_str = if provider_psk.is_empty() {
                    request_json.provider_psk
                } else {
                    provider_psk
                };
                let request = RequestCreator::create_from_hex_args(
                    request_json.user_ssk,
                    provider_psk_str.clone(),
                    rng,
                )?;
                println!(
                    "submitting request to provider psk: {}",
                    provider_psk_str
                );
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
            Command::ListRequestsUser { dummy: true } => {
                let wallet_accessor =
                    WalletAccessor::new(wallet_path.clone(), psw.clone());
                let note_hashes: Vec<BlsScalar> = wallet_accessor
                    .get_notes(blockchain_access_config)
                    .await?
                    .iter()
                    .flat_map(|n| n.nullified_by)
                    .collect();
                println!("current address has {} notes", note_hashes.len());

                let mut found_requests = vec![];
                let mut height = 0;
                loop {
                    let height_end = height + 10000;
                    let (requests, top) =
                        RequestScanner::scan_related_to_notes_in_block_range(
                            height,
                            height_end,
                            blockchain_access_config,
                            &note_hashes,
                        )
                        .await?;
                    found_requests.extend(requests);
                    if top <= height_end {
                        height = top;
                        break;
                    }
                    height = height_end;
                }
                let all_found_requests = found_requests.len();
                println!(
                    "scanned {} blocks, found {} requests",
                    height, all_found_requests,
                );
                for request in found_requests.iter() {
                    use group::GroupEncoding;
                    println!(
                        "found request rsa={}-{}",
                        hex::encode(request.rsa.R().to_bytes()),
                        hex::encode(request.rsa.pk_r().to_bytes())
                    );
                }
            }
            Command::ListRequestsLP { dummy: true } => {
                let mut reference_lp = ReferenceLP::create(lp_config)?;
                let (total_count, this_lp_count) =
                    reference_lp.scan(blockchain_access_config).await?;
                println!(
                    "found {} requests total, {} requests for this LP ",
                    total_count, this_lp_count
                );
                for request in reference_lp.requests_to_process.iter() {
                    use group::GroupEncoding;
                    println!(
                        "request to process by LP: rsa={}-{}",
                        hex::encode(request.rsa.R().to_bytes()),
                        hex::encode(request.rsa.pk_r().to_bytes())
                    );
                }
            }
            _ => (),
        }
        Ok(())
    }
}
