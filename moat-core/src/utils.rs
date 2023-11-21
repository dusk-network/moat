// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_wallet::WalletPath;
use rand::rngs::OsRng;
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password;

use zk_citadel::license::{License, SessionCookie};

use dusk_jubjub::JubJubScalar;

use crate::{
    BcInquirer, CitadelInquirer, CrsGetter, Error, LicenseCircuit, LicenseUser,
    TxAwaiter,
};
use dusk_wallet::RuskHttpClient;
use rkyv::ser::serializers::AllocSerializer;
use sha3::{Digest, Sha3_256};

use dusk_bls12_381::BlsScalar;

use dusk_plonk::prelude::*;

use std::fs::File;
use std::io::prelude::*;

static LABEL: &[u8] = b"dusk-network";

pub struct SetupHolder {
    pub prover: Prover,
    pub verifier: Verifier,
}

pub struct MoatCoreUtils {}

impl MoatCoreUtils {
    pub fn to_hash_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<16386>>,
    {
        let blob = rkyv::to_bytes::<_, 16386>(object)
            .expect("Serializing should be infallible")
            .to_vec();
        Self::blob_to_hash_hex(blob.as_slice())
    }

    pub fn blob_to_hash_hex(blob: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(blob);
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub async fn get_license_to_use(
        blockchain_access_config: &BlockchainAccessConfig,
        ssk: &SecretSpendKey,
        license_hash: String,
    ) -> Result<Option<(u64, License)>, Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        let end_height = BcInquirer::block_height(&client).await?;
        let block_heights = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_heights).await?;

        let pairs =
            CitadelInquirer::find_owned_licenses(*ssk, &mut licenses_stream)?;
        Ok(if pairs.is_empty() {
            None
        } else {
            for (pos, license) in pairs.iter() {
                if license_hash == Self::to_hash_hex(license) {
                    return Ok(Some((*pos, license.clone())));
                }
            }
            None
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn prove_and_send_use_license(
        blockchain_access_config: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        psw: &Password,
        psk_lp: &PublicSpendKey,
        psk_sp: &PublicSpendKey,
        ssk_user: &SecretSpendKey,
        challenge: &JubJubScalar,
        license: &License,
        pos: u64,
        gas_limit: u64,
        gas_price: u64,
        sh_opt: &mut Option<SetupHolder>,
        rng: &mut OsRng,
    ) -> Result<(BlsScalar, SessionCookie), Error> {
        let client =
            RuskHttpClient::new(blockchain_access_config.rusk_address.clone());

        let setup_holder = match sh_opt {
            Some(sh) => sh,
            _ => {
                let wallet_dir_path = match wallet_path.dir() {
                    Some(path) => path,
                    None => panic!(),
                };

                let prover_path = &wallet_dir_path.join("moat_prover.dat");
                let verifier_path = &wallet_dir_path.join("moat_verifier.dat");

                if prover_path.exists() && verifier_path.exists() {
                    let mut file = File::open(prover_path)?;
                    let mut prover_bytes = vec![];
                    file.read_to_end(&mut prover_bytes)?;
                    let prover = Prover::try_from_bytes(prover_bytes)?;

                    file = File::open(verifier_path)?;
                    let mut verifier_bytes = vec![];
                    file.read_to_end(&mut verifier_bytes)?;
                    let verifier = Verifier::try_from_bytes(verifier_bytes)?;

                    let sh = SetupHolder { prover, verifier };
                    *sh_opt = Some(sh);
                    sh_opt.as_ref().expect("setup holder is not empty")
                } else {
                    println!("obtaining setup");
                    let pp_vec = CrsGetter::get_crs(&client).await?;
                    let pp =
                        // SAFETY: CRS vector is checked by the hash check when it is received from the node
                        unsafe { PublicParameters::from_slice_unchecked(pp_vec.as_slice()) };
                    println!("compiling circuit");
                    let (prover, verifier) =
                        Compiler::compile::<LicenseCircuit>(&pp, LABEL)
                            .expect("Compiling circuit should succeed");

                    let mut file = File::create(prover_path)?;
                    file.write_all(prover.to_bytes().as_slice())?;

                    file = File::create(verifier_path)?;
                    file.write_all(verifier.to_bytes().as_slice())?;

                    let sh = SetupHolder { prover, verifier };
                    *sh_opt = Some(sh);
                    sh_opt.as_ref().expect("setup holder is not empty")
                }
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
            ssk_user,
            psk_lp,
            psk_sp,
            &setup_holder.prover,
            &setup_holder.verifier,
            license,
            opening,
            rng,
            challenge,
            gas_limit,
            gas_price,
        )
        .await?;
        TxAwaiter::wait_for(&client, tx_id).await?;
        Ok((tx_id, session_cookie))
    }
}
