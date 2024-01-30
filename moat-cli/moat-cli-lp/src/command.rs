// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::run_result::{
    IssueLicenseSummary, LicenseContractSummary, RequestsLPSummary, RunResult,
};
use crate::SeedableRng;
use dusk_jubjub::JubJubScalar;
use dusk_wallet::RuskHttpClient;
use moat_cli_common::Error;
use rand::rngs::StdRng;
use zk_citadel_moat::api::{MoatContext, MoatCore};
use zk_citadel_moat::license_provider::{LicenseIssuer, ReferenceLP};
use zk_citadel_moat::{BcInquirer, CitadelInquirer};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// List requests
    ListRequestsLP,
    /// Issue license
    IssueLicenseLP {
        request_hash: String,
        attr_data_bytes: String,
    },
    /// List licenses (User)
    ListLicenses,
    /// Show state
    ShowState,
}

impl Command {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        self,
        moat_context: &MoatContext,
    ) -> Result<RunResult, Error> {
        let run_result = match self {
            Command::ListRequestsLP => {
                Self::list_requests_lp(moat_context).await?
            }
            Command::IssueLicenseLP {
                request_hash,
                attr_data_bytes,
            } => {
                Self::issue_license_lp(
                    moat_context,
                    request_hash,
                    attr_data_bytes,
                )
                .await?
            }
            Command::ListLicenses => Self::list_licenses(moat_context).await?,
            Command::ShowState => Self::show_state(moat_context).await?,
        };
        Ok(run_result)
    }

    /// Command: List Requests LP
    async fn list_requests_lp(
        moat_context: &MoatContext,
    ) -> Result<RunResult, Error> {
        let (_psk, ssk) = MoatCore::get_wallet_keypair(moat_context)?;
        let mut reference_lp = ReferenceLP::create_with_ssk(&ssk)?;
        let (found_total, found_owned) = reference_lp
            .scan(&moat_context.blockchain_access_config)
            .await?;
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
        moat_context: &MoatContext,
        request_hash: String,
        attr_data_bytes: String,
    ) -> Result<RunResult, Error> {
        let attr_data = JubJubScalar::from(attr_data_bytes.parse::<u64>()?);

        let mut rng = StdRng::from_entropy();
        let (_psk, ssk) = MoatCore::get_wallet_keypair(moat_context)?;
        let mut reference_lp = ReferenceLP::create_with_ssk(&ssk)?;
        let (_total_count, _this_lp_count) = reference_lp
            .scan(&moat_context.blockchain_access_config)
            .await?;

        let request = reference_lp.get_request(&request_hash);
        Ok(match request {
            Some(request) => {
                let (tx_id, license_blob) = LicenseIssuer::issue_license(
                    &mut rng,
                    &request,
                    &attr_data,
                    moat_context,
                )
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
        moat_context: &MoatContext,
    ) -> Result<RunResult, Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
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
        moat_context: &MoatContext,
    ) -> Result<RunResult, Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        let (num_licenses, _, num_sessions) =
            CitadelInquirer::get_info(&client).await?;
        let summary = LicenseContractSummary {
            num_licenses,
            num_sessions,
        };
        Ok(RunResult::ShowState(summary))
    }
}
