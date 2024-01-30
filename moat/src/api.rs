// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::{JubJubAffine, JubJubScalar};
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_plonk::composer::{Prover, Verifier};
use dusk_wallet::{RuskHttpClient, Wallet, WalletPath};

use zk_citadel::license::{License, Request, Session, SessionCookie};

use crate::license_provider::{LicenseIssuer, ReferenceLP};
use crate::utils::MoatCoreUtils;
use crate::wallet_accessor::Password::Pwd;
use crate::wallet_accessor::{BlockchainAccessConfig, WalletAccessor};
use crate::{
    BcInquirer, CitadelInquirer, CrsGetter, LicenseCircuit, LicenseSessionId,
    RequestCreator, RequestSender, TxAwaiter,
};

use rand::rngs::OsRng;
use rand_core::{CryptoRng, RngCore};
use std::path::Path;
use std::sync::Arc;
use toml_base_config::BaseConfig;

use dusk_plonk::prelude::*;

pub use crate::Error;
use std::fs::File;
use std::io::prelude::*;

static LABEL: &[u8] = b"dusk-network";

pub struct MoatCore {}

impl MoatCore {
    /// Retrieve the keypair (psk, ssk) from the installed wallet, given the
    /// Moat Context
    pub fn get_wallet_keypair(
        moat_context: &MoatContext,
    ) -> Result<(PublicSpendKey, SecretSpendKey), Error> {
        moat_context
            .wallet
            .spending_keys(moat_context.wallet.default_address())
            .map_err(|e| Error::DuskWallet(Arc::from(e)))
    }

    /// Create and send a transaction containing a license request
    pub async fn request_license(
        psk_lp: &PublicSpendKey,
        moat_context: &MoatContext,
        rng: &mut OsRng,
    ) -> Result<(String, BlsScalar), Error> {
        let (_psk_user, ssk_user) = MoatCore::get_wallet_keypair(moat_context)?;

        let request = RequestCreator::create(&ssk_user, psk_lp, rng)?;
        let request_hash = MoatCoreUtils::to_hash_hex(&request);

        let tx_id = RequestSender::send_request(request, moat_context).await?;

        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        TxAwaiter::wait_for(&client, tx_id).await?;

        Ok((request_hash, tx_id))
    }

    /// Retrieve a vector containing all the licenses owned by a given secret
    /// key
    pub async fn get_owned_licenses(
        ssk_user: &SecretSpendKey,
        moat_context: &MoatContext,
    ) -> Result<Vec<License>, Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        let end_height = BcInquirer::block_height(&client).await?;
        let block_range = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_range.clone()).await?;

        let pairs = CitadelInquirer::find_all_licenses(&mut licenses_stream)?;
        let vk = ssk_user.view_key();
        let mut licenses = vec![];

        for (_pos, license) in pairs.into_iter() {
            if vk.owns(&license.lsa) {
                licenses.push(license);
            }
        }

        Ok(licenses)
    }

    /// Retrieve all the requests owned by the LP
    pub async fn get_owned_requests(
        ssk_lp: &SecretSpendKey,
        moat_context: &MoatContext,
    ) -> Result<Vec<Request>, Error> {
        let mut reference_lp = ReferenceLP::create_with_ssk(ssk_lp)?;
        reference_lp
            .scan(&moat_context.blockchain_access_config)
            .await?;

        Ok(reference_lp.requests_to_process)
    }

    /// Create and send a transaction containing a license for a given request
    pub async fn issue_license<R: RngCore + CryptoRng>(
        request: &Request,
        moat_context: &MoatContext,
        attr_data: &JubJubScalar,
        rng: &mut R,
    ) -> Result<String, Error> {
        let (_psk_lp, ssk_lp) = MoatCore::get_wallet_keypair(moat_context)?;
        let mut reference_lp = ReferenceLP::create_with_ssk(&ssk_lp)?;

        let (_total_count, _this_lp_count) = reference_lp
            .scan(&moat_context.blockchain_access_config)
            .await?;

        let (_tx_id, license_blob) =
            LicenseIssuer::issue_license(rng, request, attr_data, moat_context)
                .await?;
        Ok(MoatCoreUtils::blob_to_hash_hex(&license_blob))
    }

    /// Create and send a transaction containing a proof that uses a given
    /// license
    pub async fn use_license(
        moat_context: &MoatContext,
        psk_lp: &PublicSpendKey,
        psk_sp: &PublicSpendKey,
        challenge: &JubJubScalar,
        license: &License,
        rng: &mut OsRng,
    ) -> Result<Option<SessionCookie>, Error> {
        let license_hash = MoatCoreUtils::to_hash_hex(license);
        let pos_license = MoatCoreUtils::get_license_to_use(
            moat_context,
            license_hash.to_owned(),
        )
        .await?;

        match pos_license {
            Some((pos, license)) => {
                let (_tx_id, session_cookie) =
                    MoatCoreUtils::prove_and_send_use_license(
                        moat_context,
                        psk_lp,
                        psk_sp,
                        challenge,
                        &license,
                        pos,
                        rng,
                    )
                    .await?;

                Ok(Some(session_cookie))
            }
            _ => Ok(None),
        }
    }

    /// Given a session cookie, verify that it corresponds to an existing
    /// session in the Blockchain
    pub async fn verify_requested_service(
        moat_context: &MoatContext,
        psk_lp: &PublicSpendKey,
        psk_sp: &PublicSpendKey,
        session_cookie: &SessionCookie,
    ) -> Result<bool, Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );

        let pk_lp = JubJubAffine::from(*psk_lp.A());
        let pk_sp = JubJubAffine::from(*psk_sp.A());

        let session_id = LicenseSessionId {
            id: session_cookie.session_id,
        };
        let session = CitadelInquirer::get_session(&client, session_id)
            .await?
            .ok_or(Error::SessionNotFound)?;

        let session = Session::from(&session.public_inputs);
        Ok(session.verifies_ok(*session_cookie, pk_lp, pk_sp))
    }
}

pub struct MoatContext {
    pub blockchain_access_config: BlockchainAccessConfig,
    pub wallet: Wallet<WalletAccessor>,
    pub prover: Prover,
    pub verifier: Verifier,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl MoatContext {
    /// Create a new Moat Context given the required configurations
    pub async fn create<T: AsRef<str>>(
        config_path: T,
        wallet_path: T,
        wallet_password: T,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<Self, Error> {
        let wallet_path = WalletPath::from(Path::new(wallet_path.as_ref()));
        let wallet_password = Pwd(wallet_password.as_ref().to_string());

        let wallet_accessor = WalletAccessor::create(
            wallet_path.clone(),
            wallet_password.clone(),
        )?;

        let blockchain_access_config =
            BlockchainAccessConfig::load_path(config_path.as_ref())?;

        let wallet = wallet_accessor
            .get_wallet(&blockchain_access_config)
            .await?;

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

            Ok(Self {
                blockchain_access_config,
                wallet,
                prover,
                verifier,
                gas_limit,
                gas_price,
            })
        } else {
            let client = RuskHttpClient::new(
                blockchain_access_config.rusk_address.clone(),
            );

            let pp_vec = CrsGetter::get_crs(&client).await?;
            let pp =
                // SAFETY: CRS vector is checked by the hash check when it is received from the node
                unsafe { PublicParameters::from_slice_unchecked(pp_vec.as_slice()) };
            let (prover, verifier) =
                Compiler::compile::<LicenseCircuit>(&pp, LABEL)
                    .expect("Compiling circuit should succeed");

            let mut file = File::create(prover_path)?;
            file.write_all(prover.to_bytes().as_slice())?;

            file = File::create(verifier_path)?;
            file.write_all(verifier.to_bytes().as_slice())?;

            Ok(Self {
                blockchain_access_config,
                wallet,
                prover,
                verifier,
                gas_limit,
                gas_price,
            })
        }
    }
}
