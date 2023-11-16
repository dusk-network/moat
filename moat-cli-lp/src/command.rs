// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::run_result::{
    IssueLicenseSummary, LicenseContractSummary, RequestsLPSummary, RunResult,
};
use crate::SeedableRng;
use dusk_pki::SecretSpendKey;
use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::{BcInquirer, CitadelInquirer, Error};
use rand::rngs::StdRng;
use wallet_accessor::{BlockchainAccessConfig, Password};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// List requests
    ListRequestsLP,
    /// Issue license
    IssueLicenseLP { request_hash: String },
    /// List licenses (User)
    ListLicenses,
    /// Show state
    ShowState,
}

impl Command {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        self,
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        ssk: &SecretSpendKey,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<RunResult, Error> {
        let run_result = match self {
            Command::ListRequestsLP => {
                Self::list_requests_lp(blockchain_access_config, ssk).await?
            }
            Command::IssueLicenseLP { request_hash } => {
                Self::issue_license_lp(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    ssk,
                    gas_limit,
                    gas_price,
                    request_hash,
                )
                .await?
            }
            Command::ListLicenses => {
                Self::list_licenses(blockchain_access_config).await?
            }
            Command::ShowState => {
                Self::show_state(blockchain_access_config).await?
            }
        };
        Ok(run_result)
    }

    /// Command: List Requests LP
    async fn list_requests_lp(
        blockchain_access_config: &BlockchainAccessConfig,
        ssk: &SecretSpendKey,
    ) -> Result<RunResult, Error> {
        let mut reference_lp = ReferenceLP::create_with_ssk(ssk)?;
        let (found_total, found_owned) =
            reference_lp.scan(blockchain_access_config).await?;
        let summary = RequestsLPSummary {
            found_total,
            found_owned,
        };
        Ok(RunResult::RequestsLP(
            summary,
            reference_lp.requests_to_process,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    /// Command: Issue License LP
    async fn issue_license_lp(
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        ssk: &SecretSpendKey,
        gas_limit: u64,
        gas_price: u64,
        request_hash: String,
    ) -> Result<RunResult, Error> {
        let mut rng = StdRng::from_entropy();
        let mut reference_lp = ReferenceLP::create_with_ssk(ssk)?;
        let (_total_count, _this_lp_count) =
            reference_lp.scan(blockchain_access_config).await?;

        let request = reference_lp.get_request(&request_hash);
        Ok(match request {
            Some(request) => {
                let license_issuer = LicenseIssuer::new(
                    blockchain_access_config.clone(),
                    wallet_path.clone(),
                    psw.clone(),
                    gas_limit,
                    gas_price,
                );
                let (tx_id, license_blob) = license_issuer
                    .issue_license(&mut rng, &request, &reference_lp.ssk_lp)
                    .await?;
                let summary = IssueLicenseSummary {
                    request,
                    tx_id: hex::encode(tx_id.to_bytes()),
                    license_blob,
                };
                RunResult::IssueLicense(Some(summary))
            }
            _ => RunResult::IssueLicense(None),
        })
    }

    /// Command: List Licenses
    async fn list_licenses(
        blockchain_access_config: &BlockchainAccessConfig,
    ) -> Result<RunResult, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_range = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_range.clone()).await?;

        let pairs = CitadelInquirer::find_all_licenses(&mut licenses_stream)?;
        Ok(RunResult::ListLicenses(
            block_range,
            pairs.into_iter().map(|(_, l)| l).collect(),
        ))
    }

    /// Command: Show State
    async fn show_state(
        blockchain_access_config: &BlockchainAccessConfig,
    ) -> Result<RunResult, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let (num_licenses, _, num_sessions) =
            CitadelInquirer::get_info(&client).await?;
        let summary = LicenseContractSummary {
            num_licenses,
            num_sessions,
        };
        Ok(RunResult::ShowState(summary))
    }
}
