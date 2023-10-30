// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::SeedableRng;
use bytecheck::CheckBytes;
use bytes::Bytes;
use dusk_bls12_381::BlsScalar;
use dusk_bytes::DeserializableSlice;
use dusk_pki::SecretSpendKey;
use dusk_plonk::prelude::*;
use dusk_wallet::{RuskHttpClient, WalletPath};
use license_provider::{LicenseIssuer, ReferenceLP};
use moat_core::Error::InvalidQueryResponse;
use moat_core::{
    BcInquirer, CitadelInquirer, Error, JsonLoader, LicenseCircuit,
    LicenseSessionId, PayloadSender, RequestCreator, RequestJson,
    RequestScanner, RequestSender, StreamAux, TxAwaiter, LICENSE_CONTRACT_ID,
    USE_LICENSE_METHOD_NAME,
};
use rand::rngs::StdRng;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::{check_archived_root, Archive, Deserialize, Infallible, Serialize};
use sha3::{Digest, Sha3_256};
use std::path::{Path, PathBuf};
use wallet_accessor::{BlockchainAccessConfig, Password, WalletAccessor};
use zk_citadel::license::{CitadelProverParameters, License};

/// Commands that can be run against the Moat
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Command {
    /// Submit request (User)
    SubmitRequest { request_path: Option<PathBuf> },
    /// List requests (User)
    ListRequestsUser { dummy: bool },
    /// List requests (LP)
    ListRequestsLP { lp_config_path: Option<PathBuf> },
    /// Issue license (LP)
    IssueLicenseLP { lp_config_path: Option<PathBuf> },
    /// List licenses (User)
    ListLicenses { dummy: bool },
    /// Use license (User)
    UseLicense { dummy: bool },
    /// Get session (SP)
    GetSession { session_id: String },
    /// Show state
    ShowState { dummy: bool },
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

// todo: move this struct to its proper place
/// Use License Argument.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct UseLicenseArg {
    pub proof: Proof,
    pub public_inputs: Vec<BlsScalar>,
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
    ) -> Result<(), Error> {
        match self {
            Command::SubmitRequest { request_path } => {
                println!("obtained request path={:?}", request_path);
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
                let request_hash_hex = Self::to_hash_hex(&request);
                println!(
                    "submitting request to provider psk: {}",
                    request_json.provider_psk
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
                println!("request submitted: {}", request_hash_hex);
                println!();
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
                let mut total_requests = 0usize;
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
                    total_requests += total;
                    if top <= height_end {
                        height = top;
                        break;
                    }
                    height = height_end;
                }
                let owned_requests = found_requests.len();
                println!(
                    "scanned {} blocks, found {} requests, {} owned requests",
                    height, total_requests, owned_requests,
                );
                for request in found_requests.iter() {
                    println!("request: {}", Self::to_hash_hex(request));
                }
                println!();
            }
            Command::ListRequestsLP { lp_config_path } => {
                let lp_config_path = match lp_config_path {
                    Some(lp_config_path) => lp_config_path,
                    _ => PathBuf::from(lp_config),
                };
                let mut reference_lp = ReferenceLP::create(lp_config_path)?;
                let (total_count, this_lp_count) =
                    reference_lp.scan(blockchain_access_config).await?;
                println!(
                    "found {} requests total, {} requests for this LP ",
                    total_count, this_lp_count
                );
                for request in reference_lp.requests_to_process.iter() {
                    println!(
                        "request to process by LP: {}",
                        Self::to_hash_hex(request)
                    );
                }
                println!();
            }
            Command::IssueLicenseLP { lp_config_path } => {
                let mut rng = StdRng::from_entropy(); // seed_from_u64(0xbeef);
                let lp_config_path = match lp_config_path {
                    Some(lp_config_path) => lp_config_path,
                    _ => PathBuf::from(lp_config),
                };
                let mut reference_lp = ReferenceLP::create(lp_config_path)?;
                let (_total_count, _this_lp_count) =
                    reference_lp.scan(blockchain_access_config).await?;
                let request =
                    reference_lp.take_request().expect("at least one request");

                let license_issuer = LicenseIssuer::new(
                    blockchain_access_config.clone(),
                    wallet_path.clone(),
                    psw.clone(),
                    gas_limit,
                    gas_price,
                );

                println!(
                    "issuing license for request: {}",
                    Self::to_hash_hex(&request)
                );
                let tx_id = license_issuer
                    .issue_license(&mut rng, &request, &reference_lp.ssk_lp)
                    .await?;
                println!(
                    "license issuing transaction {} confirmed",
                    hex::encode(tx_id.to_bytes())
                );
                println!();
            }
            Command::ListLicenses { dummy: true } => {
                let _ = self
                    .list_licenses(
                        blockchain_access_config,
                        request_json.as_ref(),
                        true,
                    )
                    .await?;
                println!();
            }
            Command::UseLicense { dummy: true } => {
                let pos_license = self
                    .list_licenses(
                        blockchain_access_config,
                        request_json.as_ref(),
                        false,
                    )
                    .await?;
                match pos_license {
                    Some((pos, license)) => {
                        println!(
                            "using license: {}",
                            Self::to_hash_hex(&license)
                        );
                        let ssk_user = SecretSpendKey::from_slice(
                            hex::decode(
                                request_json
                                    .expect("request should be provided")
                                    .user_ssk,
                            )?
                            .as_slice(),
                        )?;
                        let session_id = Self::prove_and_send_use_license(
                            blockchain_access_config,
                            wallet_path,
                            psw,
                            lp_config,
                            ssk_user,
                            &license,
                            pos,
                            gas_limit,
                            gas_price,
                        )
                        .await?;
                        println!(
                            "license used, obtained session id: {}",
                            hex::encode(session_id.to_bytes())
                        );
                    }
                    _ => {
                        println!(
                            "No license available, please obtain a license"
                        );
                    }
                }
                println!();
            }
            Command::GetSession { session_id } => {
                let client = RuskHttpClient::new(
                    blockchain_access_config.rusk_address.clone(),
                );
                let id = LicenseSessionId {
                    id: BlsScalar::from_slice(
                        hex::decode(session_id.clone())?.as_slice(),
                    )?,
                };
                match CitadelInquirer::get_session(&client, id).await? {
                    Some(session) => {
                        println!("obtained session with id={}:", session_id);
                        println!();
                        for s in session.public_inputs.iter() {
                            println!("{}", hex::encode(s.to_bytes()));
                        }
                    }
                    _ => {
                        println!("session not found");
                    }
                }
                println!();
            }
            Command::ShowState { dummy: true } => {
                let client = RuskHttpClient::new(
                    blockchain_access_config.rusk_address.clone(),
                );
                let (num_licenses, _, num_sessions) =
                    CitadelInquirer::get_info(&client).await?;
                println!(
                    "license contract state - licenses: {}, sessions: {}",
                    num_licenses, num_sessions
                );
                println!();
            }
            _ => (),
        }
        Ok(())
    }

    async fn list_licenses(
        self,
        blockchain_access_config: &BlockchainAccessConfig,
        request_json: Option<&RequestJson>,
        ui: bool,
    ) -> Result<Option<(u64, License)>, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_heights = 0..(end_height + 1);

        if ui {
            println!(
                "getting licenses within the block height range {:?}:",
                block_heights
            );
        }
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
            if ui {
                println!("licenses not found");
            }
            None
        } else {
            if ui {
                for (_, license) in pairs.iter() {
                    println!("license: {}", Self::to_hash_hex(license))
                }
            }
            pairs.last().map(|(pos, license)| (*pos, license.clone()))
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn prove_and_send_use_license(
        blockchain_access_config: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        psw: &Password,
        lp_config: &Path,
        ssk_user: SecretSpendKey,
        license: &License,
        pos: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        // let (_, _, num_sessions) = CitadelInquirer::get_info(&client).await?;
        // let challenge = JubJubScalar::from(num_sessions as u64 + 1);
        let challenge = JubJubScalar::from(0xcafebabeu64);
        let mut rng = StdRng::seed_from_u64(0xbeef);

        println!("performing setup");
        let pp = PublicParameters::setup(1 << CAPACITY, &mut rng)
            .expect("Initializing public parameters should succeed");

        println!("compiling circuit");
        let (prover, verifier) =
            Compiler::compile::<LicenseCircuit>(&pp, LABEL)
                .expect("Compiling circuit should succeed");

        let opening = CitadelInquirer::get_merkle_opening(&client, pos)
            .await?
            .expect("Opening obtained successfully");

        let reference_lp = ReferenceLP::create(lp_config)?;

        let (cpp, sc) = CitadelProverParameters::compute_parameters(
            &ssk_user,
            license,
            &reference_lp.psk_lp,
            &reference_lp.psk_lp,
            &challenge,
            &mut rng,
            opening,
        );
        let circuit = LicenseCircuit::new(&cpp, &sc);

        println!("calculating proof");
        let (proof, public_inputs) = prover
            .prove(&mut rng, &circuit)
            .expect("Proving should succeed");

        assert!(!public_inputs.is_empty());
        let session_id = public_inputs[0];

        verifier
            .verify(&proof, &public_inputs)
            .expect("Verifying the circuit should succeed");
        println!("proof validated locally");

        let use_license_arg = UseLicenseArg {
            proof,
            public_inputs,
        };

        println!("calling license contract's use_license");
        let tx_id = PayloadSender::execute_contract_method(
            use_license_arg,
            blockchain_access_config,
            wallet_path,
            psw,
            gas_limit,
            gas_price,
            LICENSE_CONTRACT_ID,
            USE_LICENSE_METHOD_NAME,
        )
        .await?;
        println!(
            "tx {} submitted, waiting for confirmation",
            hex::encode(tx_id.to_bytes())
        );
        TxAwaiter::wait_for(&client, tx_id).await?;
        println!("tx {} confirmed", hex::encode(tx_id.to_bytes()));
        Ok(session_id)
    }

    fn to_hash_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<16386>>,
    {
        let blob = rkyv::to_bytes::<_, 16386>(object)
            .expect("type should serialize correctly")
            .to_vec();
        let mut hasher = Sha3_256::new();
        hasher.update(blob);
        let result = hasher.finalize();
        hex::encode(result)
    }
}
