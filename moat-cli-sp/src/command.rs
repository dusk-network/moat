// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::config::SPCliConfig;
use crate::run_result::{LicenseContractSummary, RunResult, SessionSummary};
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_wallet::RuskHttpClient;
use moat_core::{CitadelInquirer, Error, LicenseSessionId};
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::SessionCookie;

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Request Service (User)
    RequestService { session_cookie: String },
    /// Get session (SP)
    GetSession { session_id: String },
    /// Show state
    ShowState,
}

impl Command {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        self,
        blockchain_access_config: &BlockchainAccessConfig,
        config: &SPCliConfig,
    ) -> Result<RunResult, Error> {
        let run_result = match self {
            Command::RequestService { session_cookie } => {
                Self::request_service(&session_cookie, config).await?
            }
            Command::GetSession { session_id } => {
                Self::get_session(blockchain_access_config, session_id).await?
            }
            Command::ShowState => {
                Self::show_state(blockchain_access_config).await?
            }
        };
        Ok(run_result)
    }

    /// Command: Request Service
    async fn request_service(
        session_cookie: &str,
        config: &SPCliConfig,
    ) -> Result<RunResult, Error> {
        let bytes = hex::decode(session_cookie)?;
        let sc: SessionCookie =
            rkyv::from_bytes(bytes.as_slice()).map_err(|_| Error::DeserRkyv)?;
        println!("sc={:?}", sc);
        let psk_lp: &str = &config.psk_lp;
        println!("psk_lp={:?}", psk_lp);
        Ok(RunResult::RequestService)
    }

    /// Command: Get Session
    async fn get_session(
        blockchain_access_config: &BlockchainAccessConfig,
        session_id: String,
    ) -> Result<RunResult, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let id = LicenseSessionId {
            id: BlsScalar::from_slice(
                hex::decode(session_id.clone())?.as_slice(),
            )?,
        };
        Ok(match CitadelInquirer::get_session(&client, id).await? {
            Some(session) => {
                let mut summary = SessionSummary {
                    session_id,
                    session: vec![],
                };
                for s in session.public_inputs.iter() {
                    summary.session.push(hex::encode(s.to_bytes()));
                }
                RunResult::GetSession(Some(summary))
            }
            _ => RunResult::GetSession(None),
        })
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
