// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::config::SPCliConfig;
use crate::run_result::{
    LicenseContractSummary, RunResult, ServiceRequestSummery, SessionSummary,
};
use crate::Error;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_jubjub::JubJubAffine;
use dusk_pki::PublicSpendKey;
use dusk_wallet::RuskHttpClient;
use moat_core::{CitadelInquirer, LicenseSessionId};
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::{Session, SessionCookie};

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
                Self::request_service(
                    blockchain_access_config,
                    &session_cookie,
                    config,
                )
                .await?
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
        blockchain_access_config: &BlockchainAccessConfig,
        session_cookie: &str,
        config: &SPCliConfig,
    ) -> Result<RunResult, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());

        let bytes = hex::decode(session_cookie)
            .map_err(|_| Error::InvalidEntry("session cookie".into()))?;
        let sc: SessionCookie = rkyv::from_bytes(bytes.as_slice())
            .map_err(|_| Error::InvalidEntry("session cookie".into()))?;
        let psk_lp: &str = &config.psk_lp;
        let psk_lp_bytes = hex::decode(psk_lp.as_bytes()).map_err(|_| {
            Error::InvalidConfigValue("license provider psk".into())
        })?;
        let psk_lp = PublicSpendKey::from_slice(psk_lp_bytes.as_slice())
            .map_err(|_| {
                Error::InvalidConfigValue("license provider psk".into())
            })?;
        let pk_lp = JubJubAffine::from(*psk_lp.A());

        let session_id = LicenseSessionId { id: sc.session_id };
        let session = CitadelInquirer::get_session(&client, session_id)
            .await?
            .ok_or(Error::NotFound("Session not found".into()))?;

        let session = Session::from(&session.public_inputs);
        let granted = session.verifies_ok(sc, pk_lp);
        println!("session id={}", hex::encode(session_id.id.to_bytes()));
        let service_request_summary = ServiceRequestSummery {
            service_granted: granted,
        };
        Ok(RunResult::RequestService(service_request_summary))
    }

    /// Command: Get Session
    async fn get_session(
        blockchain_access_config: &BlockchainAccessConfig,
        session_id: String,
    ) -> Result<RunResult, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let session_id_bytes = hex::decode(session_id.clone())
            .map_err(|_| Error::InvalidEntry("session id".into()))?;
        let id = LicenseSessionId {
            id: BlsScalar::from_slice(session_id_bytes.as_slice())
                .map_err(|_| Error::InvalidEntry("session id".into()))?,
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
