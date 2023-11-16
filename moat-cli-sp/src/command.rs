// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::config::SPCliConfig;
use crate::run_result::{LicenseContractSummary, RunResult, SessionSummary};
use crate::CliError;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_jubjub::{JubJubAffine, GENERATOR_EXTENDED, GENERATOR_NUMS_EXTENDED};
use dusk_pki::PublicSpendKey;
use dusk_poseidon::sponge;
use dusk_wallet::RuskHttpClient;
use moat_core::{CitadelInquirer, Error, LicenseSession, LicenseSessionId};
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
    ) -> Result<RunResult, CliError> {
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
    ) -> Result<RunResult, CliError> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());

        let bytes = hex::decode(session_cookie)?;
        let sc: SessionCookie = rkyv::from_bytes(bytes.as_slice())
            .map_err(|_| Error::InvalidData("session cookie".into()))?;
        println!("sc={:?}", sc);
        let psk_lp: &str = &config.psk_lp;
        println!("psk_lp={:?}", psk_lp);
        let psk_lp_bytes = hex::decode(psk_lp.as_bytes())?;
        let psk_lp = PublicSpendKey::from_slice(psk_lp_bytes.as_slice())?;
        let pk_lp = JubJubAffine::from(*psk_lp.A());

        let session_id = LicenseSessionId { id: sc.session_id };
        let session = CitadelInquirer::get_session(&client, session_id)
            .await?
            .ok_or(CliError::NotFound("Session not found".into()))?;

        println!("session found");
        let b: bool = Self::verify_session_cookie(&sc, pk_lp, &session);
        println!("session verified: {}", b);
        Ok(RunResult::RequestService)
    }

    // todo: move this function somewhere else
    // todo: because of asserts we cannot use Session::verify from zk_citadel
    // here
    // todo: need to provide a non-panicking version of Session:verify
    // in zk_citadel
    fn verify_session_cookie(
        sc: &SessionCookie,
        pk_lp: JubJubAffine,
        session: &LicenseSession,
    ) -> bool {
        let session = Session::from(&session.public_inputs);

        // assert_eq!(pk_lp, sc.pk_lp);
        if pk_lp != sc.pk_lp {
            return false;
        }

        let session_hash =
            sponge::hash(&[sc.pk_sp.get_u(), sc.pk_sp.get_v(), sc.r]);
        // assert_eq!(session_hash, self.session_hash);
        if session_hash != session.session_hash {
            return false;
        }

        let com_0 = sponge::hash(&[pk_lp.get_u(), pk_lp.get_v(), sc.s_0]);
        // assert_eq!(com_0, self.com_0);
        if com_0 != session.com_0 {
            return false;
        }

        let com_1 = (GENERATOR_EXTENDED * sc.attr_data)
            + (GENERATOR_NUMS_EXTENDED * sc.s_1);
        // assert_eq!(com_1, self.com_1);
        if com_1 != session.com_1 {
            return false;
        }

        let com_2 =
            (GENERATOR_EXTENDED * sc.c) + (GENERATOR_NUMS_EXTENDED * sc.s_2);
        // assert_eq!(com_2, self.com_2);
        if com_2 != session.com_2 {
            return false;
        }
        true
    }

    /// Command: Get Session
    async fn get_session(
        blockchain_access_config: &BlockchainAccessConfig,
        session_id: String,
    ) -> Result<RunResult, CliError> {
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
