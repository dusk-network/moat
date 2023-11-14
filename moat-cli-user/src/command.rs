// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::interactor::SetupHolder;
use crate::run_result::{
    LicenseContractSummary, RunResult, SubmitRequestSummary,
    UseLicenseSummary,
};
use crate::SeedableRng;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_plonk::prelude::*;
use dusk_wallet::{RuskHttpClient, WalletPath};
use moat_core::{
    BcInquirer, CitadelInquirer, CrsGetter, Error, LicenseCircuit, LicenseUser,
    RequestCreator, RequestSender, TxAwaiter,
};
use rand::rngs::StdRng;
use wallet_accessor::{BlockchainAccessConfig, Password};
use zk_citadel::license::{License, SessionCookie};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Submit request (User)
    SubmitRequest { psk_lp_bytes: String },
    /// List licenses (User)
    ListLicenses,
    /// Use license (User)
    UseLicense { license_hash: String },
    /// Request Service (User)
    RequestService { session_cookie: String },
    /// Show state
    ShowState,
}

static LABEL: &[u8] = b"dusk-network";

impl Command {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        self,
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
        ssk: SecretSpendKey,
        setup_holder: &mut Option<SetupHolder>,
    ) -> Result<RunResult, Error> {
        let run_result = match self {
            Command::SubmitRequest { psk_lp_bytes }=> {
                Self::submit_request(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    gas_limit,
                    gas_price,
                    ssk,
                    psk_lp_bytes,
                )
                .await?
            }
            Command::ListLicenses => {
                Self::list_licenses(blockchain_access_config, ssk)
                    .await?
            }
            Command::UseLicense { license_hash } => {
                Self::use_license(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    gas_limit,
                    gas_price,
                    ssk,
                    setup_holder,
                    license_hash,
                )
                .await?
            }
            Command::RequestService { session_cookie: _ } => {
                println!("Off-chain request service to be placed here");
                RunResult::Empty
            }
            Command::ShowState => {
                Self::show_state(blockchain_access_config).await?
            }
        };
        Ok(run_result)
    }

    /// Command: Submit Request
    #[allow(non_snake_case)]
    async fn submit_request(
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
        ssk: SecretSpendKey,
        psk_lp_bytes: String,
    ) -> Result<RunResult, Error> {
        let A = JubJubExtended::from(JubJubAffine::from_slice(&psk_lp_bytes.clone().into_bytes()[..32]).unwrap());
        let B = JubJubExtended::from(JubJubAffine::from_slice(&psk_lp_bytes.clone().into_bytes()[32..]).unwrap());
        let psk_lp = PublicSpendKey::new(A, B);
        
        let rng = &mut StdRng::from_entropy(); // seed_from_u64(0xcafe);
        let request = RequestCreator::create(
            &ssk,
            &psk_lp,
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
            psk_lp: psk_lp_bytes,
            tx_id: hex::encode(tx_id.to_bytes()),
            request_hash,
        };
        Ok(RunResult::SubmitRequest(summary))
    }

    /// Command: List Licenses
    async fn list_licenses(
        blockchain_access_config: &BlockchainAccessConfig,
        ssk: SecretSpendKey,
    ) -> Result<RunResult, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_range = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_range.clone()).await?;

        let ssk_user = ssk;

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
        wallet_path: &WalletPath,
        psw: &Password,
        blockchain_access_config: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
        ssk: SecretSpendKey,
        setup_holder: &mut Option<SetupHolder>,
        license_hash: String,
    ) -> Result<RunResult, Error> {
        let pos_license = Self::get_license_to_use(
            blockchain_access_config,
            ssk,
            license_hash.clone(),
        )
        .await?;
        Ok(match pos_license {
            Some((pos, license)) => {
                println!("using license: {}", RunResult::to_hash_hex(&license));
                let ssk_user = ssk;

                let psk_lp_bytes = "136d747ff489bd06077f937508b9237ac093ff868dc2e232ab3af0ecd038873288560dbd8aa851e055bc408ebeb89509b26eb6e34b4b43214de467e3ef09594e".to_string();

                let A = JubJubExtended::from(JubJubAffine::from_slice(&psk_lp_bytes.clone().into_bytes()[..32]).unwrap());
                let B = JubJubExtended::from(JubJubAffine::from_slice(&psk_lp_bytes.clone().into_bytes()[32..]).unwrap());
                let psk_lp = PublicSpendKey::new(A, B);

                let (tx_id, session_cookie) = Self::prove_and_send_use_license(
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
                let summary = UseLicenseSummary {
                    license_blob: RunResult::to_blob(&license),
                    tx_id: hex::encode(tx_id.to_bytes()),
                    user_attr: hex::encode(session_cookie.attr_data.to_bytes()),
                    session_id: hex::encode(
                        session_cookie.session_id.to_bytes(),
                    ),
                    session_cookie: RunResult::to_blob_hex(&session_cookie),
                };
                RunResult::UseLicense(Some(summary))
            }
            _ => RunResult::UseLicense(None),
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
        ssk: SecretSpendKey,
        license_hash: String,
    ) -> Result<Option<(u64, License)>, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_heights = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_heights).await?;

        let ssk_user = ssk;

        let pairs = CitadelInquirer::find_owned_licenses(
            ssk_user,
            &mut licenses_stream,
        )?;
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
    ) -> Result<(BlsScalar, SessionCookie), Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        // let (_, _, num_sessions) = CitadelInquirer::get_info(&client).await?;
        // let challenge = JubJubScalar::from(num_sessions as u64 + 1);
        let challenge = JubJubScalar::from(0xcafebabeu64);
        let mut rng = StdRng::seed_from_u64(0xbeef);

        let setup_holder = match sh_opt {
            Some(sh) => sh,
            _ => {
                println!("obtaining setup");
                let pp_vec = CrsGetter::get_crs(&client).await?;
                let pp =
                    // SAFETY: CRS vector is checked by the hash check when it is received from the node
                    unsafe { PublicParameters::from_slice_unchecked(pp_vec.as_slice()) };
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
        Ok((tx_id, session_cookie))
    }
}
