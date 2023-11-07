// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::interactor::SetupHolder;
use crate::run_result::{
    IssueLicenseSummary, LicenseContractSummary, RequestsLPSummary,
    RequestsSummary, RunResult, SessionSummary, SubmitRequestSummary,
};
use crate::SeedableRng;
use bytes::Bytes;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_plonk::prelude::*;
use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::Error::InvalidQueryResponse;
use moat_core::{
    BcInquirer, CitadelInquirer, Error, JsonLoader, LicenseCircuit,
    LicenseSessionId, LicenseUser, RequestCreator, RequestJson, RequestScanner,
    RequestSender, StreamAux, TxAwaiter,
};
use rand::rngs::StdRng;
use rkyv::{check_archived_root, Deserialize, Infallible};
use std::path::{Path, PathBuf};
use wallet_accessor::{BlockchainAccessConfig, Password, WalletAccessor};
use zk_citadel::license::License;

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Submit request (User)
    SubmitRequest { request_path: Option<PathBuf> },
    /// List requests (User)
    ListRequestsUser,
    /// List requests (LP)
    ListRequestsLP { lp_config_path: Option<PathBuf> },
    /// Issue license (LP)
    IssueLicenseLP {
        lp_config_path: Option<PathBuf>,
        request_hash: String,
    },
    /// List licenses (User)
    ListLicenses { request_path: Option<PathBuf> },
    /// Use license (User)
    UseLicense {
        request_path: Option<PathBuf>,
        license_hash: String,
    },
    /// Request Service (User)
    RequestService { session_cookie: String },
    /// Get session (SP)
    GetSession { session_id: String },
    /// Show state
    ShowState,
}

// todo: move this function somewhere else
/// Deserializes license, panics if deserialization fails.
fn deserialise_license(v: &Vec<u8>) -> License {
    let response_data = check_archived_root::<License>(v.as_slice())
        .map_err(|_| {
            InvalidQueryResponse(Box::from("rkyv deserialization error"))
        })
        .expect("License should deserialize correctly");
    let license: License = response_data
        .deserialize(&mut Infallible)
        .expect("Infallible");
    license
}

// todo: move this function somewhere else
/// Finds owned license in a stream of licenses.
/// It searches in a reverse order to return a newest license.
fn find_owned_licenses(
    ssk_user: SecretSpendKey,
    stream: &mut (impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
              + std::marker::Unpin),
) -> Result<Vec<(u64, License)>, Error> {
    const ITEM_LEN: usize = CitadelInquirer::GET_LICENSES_ITEM_LEN;
    let mut pairs = vec![];
    loop {
        let r = StreamAux::find_item::<(u64, Vec<u8>), ITEM_LEN>(
            |(_, lic_vec)| {
                let license = deserialise_license(lic_vec);
                Ok(ssk_user.view_key().owns(&license.lsa))
            },
            stream,
        );
        if r.is_err() {
            break;
        }
        let (pos, lic_ser) = r?;
        pairs.push((pos, deserialise_license(&lic_ser)))
    }
    Ok(pairs)
}

// todo: move this function somewhere else and possibly merge with
// find_owned_licenses
/// Finds owned license in a stream of licenses.
/// It searches in a reverse order to return a newest license.
fn find_all_licenses(
    stream: &mut (impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
              + std::marker::Unpin),
) -> Result<Vec<(u64, License)>, Error> {
    const ITEM_LEN: usize = CitadelInquirer::GET_LICENSES_ITEM_LEN;
    let mut pairs = vec![];
    loop {
        let r = StreamAux::find_item::<(u64, Vec<u8>), ITEM_LEN>(
            |_| Ok(true),
            stream,
        );
        if r.is_err() {
            break;
        }
        let (pos, lic_ser) = r?;
        pairs.push((pos, deserialise_license(&lic_ser)))
    }
    Ok(pairs)
}

// todo: move these consts to their proper place
static LABEL: &[u8] = b"dusk-network";
const CAPACITY: usize = 17; // capacity required for the setup

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
        setup_holder: &mut Option<SetupHolder>,
    ) -> Result<RunResult, Error> {
        let run_result = match self {
            Command::SubmitRequest { request_path } => {
                Self::submit_request(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    gas_limit,
                    gas_price,
                    request_json,
                    request_path,
                )
                .await?
            }
            Command::ListRequestsUser => {
                Self::list_requests(wallet_path, psw, blockchain_access_config)
                    .await?
            }
            Command::ListRequestsLP { lp_config_path } => {
                Self::list_requests_lp(
                    blockchain_access_config,
                    lp_config,
                    lp_config_path,
                )
                .await?
            }
            Command::IssueLicenseLP {
                lp_config_path,
                request_hash,
            } => {
                Self::issue_license_lp(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    lp_config,
                    gas_limit,
                    gas_price,
                    lp_config_path,
                    request_hash,
                )
                .await?
            }
            Command::ListLicenses { request_path } => {
                Self::list_licenses(
                    blockchain_access_config,
                    request_json,
                    request_path,
                )
                .await?
            }
            Command::UseLicense {
                request_path,
                license_hash,
            } => {
                Self::use_license(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    gas_limit,
                    gas_price,
                    request_json,
                    setup_holder,
                    request_path,
                    license_hash,
                )
                .await?
            }
            Command::RequestService { session_cookie: _ } => {
                println!("Off-chain request service to be placed here");
                RunResult::Empty
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

    /// Command: Submit Request
    async fn submit_request(
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
        request_json: Option<RequestJson>,
        request_path: Option<PathBuf>,
    ) -> Result<RunResult, Error> {
        let request_json = match request_path {
            Some(request_path) => RequestJson::from_file(request_path)?,
            _ => request_json.expect("request should be provided"),
        };
        let rng = &mut StdRng::from_entropy(); // seed_from_u64(0xcafe);
        let request = RequestCreator::create_from_hex_args(
            request_json.user_ssk,
            request_json.provider_psk.clone(),
            rng,
        )?;
        let request_hash = RunResult::to_hash_hex(&request);
        let tx_id = RequestSender::send_request(
            request,
            blockchain_access_config,
            wallet_path,
            psw,
            gas_limit,
            gas_price,
        )
        .await?;
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        TxAwaiter::wait_for(&client, tx_id).await?;
        let summary = SubmitRequestSummary {
            psk_lp: request_json.provider_psk,
            tx_id: hex::encode(tx_id.to_bytes()),
            request_hash,
        };
        Ok(RunResult::SubmitRequest(summary))
    }

    /// Command: List Requests
    async fn list_requests(
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
    ) -> Result<RunResult, Error> {
        let wallet_accessor =
            WalletAccessor::new(wallet_path.clone(), psw.clone());
        let note_hashes: Vec<BlsScalar> = wallet_accessor
            .get_notes(blockchain_access_config)
            .await?
            .iter()
            .flat_map(|n| n.nullified_by)
            .collect();

        let mut found_requests = vec![];
        let mut height = 0;
        let mut found_total = 0usize;
        loop {
            let height_end = height + 10000;
            let (requests, top, total) =
                RequestScanner::scan_related_to_notes_in_block_range(
                    height,
                    height_end,
                    blockchain_access_config,
                    &note_hashes,
                )
                .await?;
            found_requests.extend(requests);
            found_total += total;
            if top <= height_end {
                height = top;
                break;
            }
            height = height_end;
        }
        let found_owned = found_requests.len();
        let summary = RequestsSummary {
            height,
            found_total,
            found_owned,
        };
        let run_result = RunResult::Requests(summary, found_requests);
        Ok(run_result)
    }

    /// Command: List Requests LP
    async fn list_requests_lp(
        blockchain_access_config: &BlockchainAccessConfig,
        lp_config: &Path,
        lp_config_path: Option<PathBuf>,
    ) -> Result<RunResult, Error> {
        let lp_config_path = match lp_config_path {
            Some(lp_config_path) => lp_config_path,
            _ => PathBuf::from(lp_config),
        };
        let mut reference_lp = ReferenceLP::create(lp_config_path)?;
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
        lp_config: &Path,
        gas_limit: u64,
        gas_price: u64,
        lp_config_path: Option<PathBuf>,
        request_hash: String,
    ) -> Result<RunResult, Error> {
        let mut rng = StdRng::from_entropy(); // seed_from_u64(0xbeef);
        let lp_config_path = match lp_config_path {
            Some(lp_config_path) => lp_config_path,
            _ => PathBuf::from(lp_config),
        };
        let mut reference_lp = ReferenceLP::create(lp_config_path)?;
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
        request_json: Option<RequestJson>,
        request_path: Option<PathBuf>,
    ) -> Result<RunResult, Error> {
        let request_json = match request_path {
            Some(request_path) => RequestJson::from_file(request_path)?,
            _ => request_json.expect("request should be provided"),
        };

        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_range = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_range.clone()).await?;

        let ssk_user = SecretSpendKey::from_slice(
            hex::decode(request_json.user_ssk.clone())?.as_slice(),
        )?;

        let pairs = find_all_licenses(&mut licenses_stream)?;
        let vk = ssk_user.view_key();
        let mut licenses = vec![];
        for (_pos, license) in pairs.into_iter() {
            let is_owned = vk.owns(&license.lsa);
            licenses.push((license, is_owned));
        }
        Ok(RunResult::ListLicenses(block_range, licenses))
    }

    #[allow(clippy::too_many_arguments)]
    /// Command: Use License
    async fn use_license(
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
        request_json: Option<RequestJson>,
        setup_holder: &mut Option<SetupHolder>,
        request_path: Option<PathBuf>,
        license_hash: String,
    ) -> Result<RunResult, Error> {
        let request_json = match request_path {
            Some(request_path) => RequestJson::from_file(request_path)?,
            _ => request_json.expect("request should be provided"),
        };
        let pos_license = Self::get_license_to_use(
            blockchain_access_config,
            Some(&request_json),
            license_hash.clone(),
        )
        .await?;
        match pos_license {
            Some((pos, license)) => {
                println!("using license: {}", RunResult::to_hash_hex(&license));
                let ssk_user = SecretSpendKey::from_slice(
                    hex::decode(request_json.user_ssk)?.as_slice(),
                )?;
                let psk_lp = PublicSpendKey::from_slice(
                    hex::decode(request_json.provider_psk)?.as_slice(),
                )?;
                let _session_id = Self::prove_and_send_use_license(
                    blockchain_access_config,
                    wallet_path,
                    psw,
                    psk_lp,
                    ssk_user,
                    &license,
                    pos,
                    gas_limit,
                    gas_price,
                    setup_holder,
                )
                .await?;
            }
            _ => {
                println!("Please obtain a license");
            }
        }
        Ok(RunResult::Empty)
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

    async fn get_license_to_use(
        blockchain_access_config: &BlockchainAccessConfig,
        request_json: Option<&RequestJson>,
        license_hash: String,
    ) -> Result<Option<(u64, License)>, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_heights = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_heights).await?;

        let ssk_user = SecretSpendKey::from_slice(
            hex::decode(
                request_json
                    .expect("request should be provided")
                    .user_ssk
                    .clone(),
            )?
            .as_slice(),
        )?;

        let pairs = find_owned_licenses(ssk_user, &mut licenses_stream)?;
        Ok(if pairs.is_empty() {
            None
        } else {
            for (pos, license) in pairs.iter() {
                if license_hash == RunResult::to_hash_hex(license) {
                    return Ok(Some((*pos, license.clone())));
                }
            }
            None
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn prove_and_send_use_license(
        blockchain_access_config: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        psw: &Password,
        psk_lp: PublicSpendKey,
        ssk_user: SecretSpendKey,
        license: &License,
        pos: u64,
        gas_limit: u64,
        gas_price: u64,
        sh_opt: &mut Option<SetupHolder>,
    ) -> Result<BlsScalar, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        // let (_, _, num_sessions) = CitadelInquirer::get_info(&client).await?;
        // let challenge = JubJubScalar::from(num_sessions as u64 + 1);
        let challenge = JubJubScalar::from(0xcafebabeu64);
        let mut rng = StdRng::seed_from_u64(0xbeef);

        let setup_holder = match sh_opt {
            Some(sh) => sh,
            _ => {
                println!("performing setup");
                let pp = PublicParameters::setup(1 << CAPACITY, &mut rng)
                    .expect("Initializing public parameters should succeed");
                println!("compiling circuit");
                let (prover, verifier) =
                    Compiler::compile::<LicenseCircuit>(&pp, LABEL)
                        .expect("Compiling circuit should succeed");
                let sh = SetupHolder {
                    pp,
                    prover,
                    verifier,
                };
                *sh_opt = Some(sh);
                sh_opt.as_ref().unwrap()
            }
        };

        let opening = CitadelInquirer::get_merkle_opening(&client, pos)
            .await?
            .expect("Opening obtained successfully");

        println!(
            "calculating proof and calling license contract's use_license"
        );
        let (tx_id, session_cookie) = LicenseUser::prove_and_use_license(
            blockchain_access_config,
            wallet_path,
            psw,
            &ssk_user,
            &psk_lp,
            &setup_holder.prover,
            &setup_holder.verifier,
            license,
            opening,
            &mut rng,
            &challenge,
            gas_limit,
            gas_price,
        )
        .await?;
        TxAwaiter::wait_for(&client, tx_id).await?;
        println!(
            "use license executing transaction {} confirmed",
            hex::encode(tx_id.to_bytes())
        );
        println!();
        println!("license {} used", RunResult::to_hash_hex(license),);
        println!();
        println!(
            "session cookie: {}",
            RunResult::to_blob_hex(&session_cookie)
        );
        println!();
        println!(
            "user attributes: {}",
            hex::encode(session_cookie.attr.to_bytes())
        );
        println!(
            "session id: {}",
            hex::encode(session_cookie.session_id.to_bytes())
        );
        Ok(session_cookie.session_id)
    }
}
