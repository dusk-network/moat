// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::run_result::{
    LicenseContractSummary, RunResult, SubmitRequestSummary, UseLicenseSummary,
};
use crate::SeedableRng;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_pki::PublicSpendKey;
use dusk_plonk::prelude::*;
use dusk_wallet::RuskHttpClient;
use moat_cli_common::Error;
use rand::rngs::{OsRng, StdRng};
use zk_citadel::license::{License, SessionCookie};
use zk_citadel_moat::{
    BcInquirer, CitadelInquirer, LicenseUser, MoatCoreUtils, RequestCreator,
    RequestSender, TxAwaiter,
};

use zk_citadel_moat::api::{MoatContext, MoatCore};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Submit request (User)
    SubmitRequest { psk_lp_bytes: String },
    /// List licenses (User)
    ListLicenses,
    /// Use license (User)
    UseLicense {
        license_hash: String,
        psk_lp_bytes: String,
        psk_sp_bytes: String,
        challenge_bytes: String,
    },
    /// Request Service (User)
    RequestService { session_cookie: String },
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
            Command::SubmitRequest { psk_lp_bytes } => {
                Self::submit_request(moat_context, psk_lp_bytes).await?
            }
            Command::ListLicenses => Self::list_licenses(moat_context).await?,
            Command::UseLicense {
                license_hash,
                psk_lp_bytes,
                psk_sp_bytes,
                challenge_bytes,
            } => {
                Self::use_license(
                    moat_context,
                    psk_lp_bytes,
                    psk_sp_bytes,
                    challenge_bytes,
                    license_hash,
                )
                .await?
            }
            Command::RequestService { session_cookie: _ } => {
                println!("Off-chain request service to be placed here");
                RunResult::Empty
            }
            Command::ShowState => Self::show_state(moat_context).await?,
        };
        Ok(run_result)
    }

    /// Command: Submit Request
    #[allow(non_snake_case)]
    async fn submit_request<T: AsRef<str>>(
        moat_context: &MoatContext,
        psk_lp_bytes: T,
    ) -> Result<RunResult, Error> {
        let psk_lp_bytes_formatted =
            bs58::decode(psk_lp_bytes.as_ref()).into_vec()?;
        let psk_lp =
            PublicSpendKey::from_slice(psk_lp_bytes_formatted.as_slice())?;

        let rng = &mut StdRng::from_entropy();
        let (_psk, ssk) = MoatCore::get_wallet_keypair(moat_context)?;
        let request = RequestCreator::create(&ssk, &psk_lp, rng)?;
        let request_hash = MoatCoreUtils::to_hash_hex(&request);
        let tx_id = RequestSender::send_request(request, moat_context).await?;
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        TxAwaiter::wait_for(&client, tx_id).await?;

        let summary = SubmitRequestSummary {
            psk_lp: psk_lp_bytes.as_ref().to_string(),
            tx_id: hex::encode(tx_id.to_bytes()),
            request_hash,
        };
        Ok(RunResult::SubmitRequest(summary))
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

        let (_psk_user, ssk_user) = MoatCore::get_wallet_keypair(moat_context)?;

        let pairs = CitadelInquirer::find_all_licenses(&mut licenses_stream)?;
        let vk = ssk_user.view_key();
        let mut licenses = vec![];
        for (_pos, license) in pairs.into_iter() {
            let is_owned = vk.owns(&license.lsa);
            licenses.push((license, is_owned));
        }
        Ok(RunResult::ListLicenses(block_range, licenses))
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(non_snake_case)]
    /// Command: Use License
    async fn use_license(
        moat_context: &MoatContext,
        psk_lp_bytes: String,
        psk_sp_bytes: String,
        challenge_bytes: String,
        license_hash: String,
    ) -> Result<RunResult, Error> {
        let pos_license =
            Self::get_license_to_use(moat_context, license_hash.clone())
                .await?;
        Ok(match pos_license {
            Some((pos, license)) => {
                println!(
                    "using license: {}",
                    MoatCoreUtils::to_hash_hex(&license)
                );
                let challenge =
                    JubJubScalar::from(challenge_bytes.parse::<u64>()?);

                let psk_lp_bytes = bs58::decode(&psk_lp_bytes).into_vec()?;
                let psk_lp =
                    PublicSpendKey::from_slice(psk_lp_bytes.as_slice())?;

                let psk_sp_bytes = bs58::decode(&psk_sp_bytes).into_vec()?;
                let psk_sp =
                    PublicSpendKey::from_slice(psk_sp_bytes.as_slice())?;

                let (tx_id, session_cookie) = Self::prove_and_send_use_license(
                    moat_context,
                    psk_lp,
                    psk_sp,
                    challenge,
                    &license,
                    pos,
                )
                .await?;
                let summary = UseLicenseSummary {
                    license_blob: MoatCoreUtils::to_blob(&license),
                    tx_id: hex::encode(tx_id.to_bytes()),
                    user_attr: hex::encode(session_cookie.attr_data.to_bytes()),
                    session_id: hex::encode(
                        session_cookie.session_id.to_bytes(),
                    ),
                    session_cookie: MoatCoreUtils::to_blob_hex(&session_cookie),
                };
                RunResult::UseLicense(Some(summary))
            }
            _ => RunResult::UseLicense(None),
        })
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

    async fn get_license_to_use(
        moat_context: &MoatContext,
        license_hash: String,
    ) -> Result<Option<(u64, License)>, Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        let end_height = BcInquirer::block_height(&client).await?;
        let block_heights = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_heights).await?;

        let pairs = CitadelInquirer::find_owned_licenses(
            moat_context,
            &mut licenses_stream,
        )?;
        Ok(if pairs.is_empty() {
            None
        } else {
            for (pos, license) in pairs.iter() {
                if license_hash == MoatCoreUtils::to_hash_hex(license) {
                    return Ok(Some((*pos, license.clone())));
                }
            }
            None
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn prove_and_send_use_license(
        moat_context: &MoatContext,
        psk_lp: PublicSpendKey,
        psk_sp: PublicSpendKey,
        challenge: JubJubScalar,
        license: &License,
        pos: u64,
    ) -> Result<(BlsScalar, SessionCookie), Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        // let (_, _, num_sessions) = CitadelInquirer::get_info(&client).await?;
        // let challenge = JubJubScalar::from(num_sessions as u64 + 1);

        let opening = CitadelInquirer::get_merkle_opening(&client, pos)
            .await?
            .expect("Opening obtained successfully");

        println!("getting prover");
        let prover = MoatCore::get_prover(moat_context).await?;

        println!(
            "calculating proof and calling license contract's use_license"
        );
        let (tx_id, session_cookie) = LicenseUser::prove_and_use_license(
            &prover,
            moat_context,
            &psk_lp,
            &psk_sp,
            license,
            opening,
            &mut OsRng,
            &challenge,
        )
        .await?;
        TxAwaiter::wait_for(&client, tx_id).await?;
        Ok((tx_id, session_cookie))
    }
}
