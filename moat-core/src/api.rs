// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::{JubJubAffine, JubJubScalar};
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_wallet::{RuskHttpClient, Wallet, WalletPath};

use zk_citadel::license::{Session, SessionCookie};

use crate::license_provider::{LicenseIssuer, ReferenceLP};
use crate::utils::MoatCoreUtils;
use crate::{
    CitadelInquirer, Error, LicenseSessionId, RequestCreator, RequestSender,
    TxAwaiter,
};
use wallet_accessor::Password::{self, Pwd};
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};

use rand::rngs::OsRng;
use std::path::Path;
use std::sync::Arc;
use toml_base_config::BaseConfig;

pub struct MoatCore {}

impl MoatCore {
    /// Retrieve the keypair (psk, ssk) from the installed wallet, given the
    /// Moat Context
    pub fn get_wallet_keypair(
        moat_context: &MoatContext,
    ) -> Result<(PublicSpendKey, SecretSpendKey), Error> {
        // We access the wallet to get the key pair
        println!("wallet_path={}", &moat_context.wallet_path);
        let wallet_accessor = WalletAccessor::create(
            moat_context.wallet_path.clone(),
            moat_context.wallet_password.clone(),
        )?;
        let wallet = Wallet::from_file(wallet_accessor)?;

        wallet
            .spending_keys(wallet.default_address())
            .map_err(|e| Error::DuskWallet(Arc::from(e)))
    }

    /// Create and send a transaction containing a license request
    pub async fn request_license(
        ssk_user: &SecretSpendKey,
        psk_lp: &PublicSpendKey,
        moat_context: &MoatContext,
        rng: &mut OsRng,
    ) -> Result<String, Error> {
        let request = RequestCreator::create(ssk_user, psk_lp, rng)?;
        let request_hash = MoatCoreUtils::to_hash_hex(&request);

        let tx_id = RequestSender::send_request(
            request,
            &moat_context.blockchain_access_config,
            &moat_context.wallet_path,
            &moat_context.wallet_password,
            moat_context.gas_limit,
            moat_context.gas_price,
        )
        .await?;

        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        TxAwaiter::wait_for(&client, tx_id).await?;

        Ok(request_hash)
    }

    /// Create and send a transaction containing a license for a given request
    pub async fn issue_license(
        request_hash: &String,
        ssk_lp: &SecretSpendKey,
        moat_context: &MoatContext,
        attr_data: &JubJubScalar,
        rng: &mut OsRng,
    ) -> Result<String, Error> {
        let mut reference_lp = ReferenceLP::create_with_ssk(ssk_lp)?;

        let (_total_count, _this_lp_count) = reference_lp
            .scan(&moat_context.blockchain_access_config)
            .await?;

        let request = reference_lp.get_request(request_hash);

        match request {
            Some(request) => {
                let license_issuer = LicenseIssuer::new(
                    moat_context.blockchain_access_config.clone(),
                    moat_context.wallet_path.clone(),
                    moat_context.wallet_password.clone(),
                    moat_context.gas_limit,
                    moat_context.gas_price,
                );
                let (_tx_id, license_blob) = license_issuer
                    .issue_license(
                        rng,
                        &request,
                        &reference_lp.ssk_lp,
                        attr_data,
                    )
                    .await?;
                Ok(MoatCoreUtils::blob_to_hash_hex(&license_blob))
            }
            _ => Ok("nothing to do".to_string()),
        }
    }

    /// Create and send a transaction containing a proof that uses a given
    /// license
    pub async fn use_license(
        moat_context: &MoatContext,
        psk_lp: &PublicSpendKey,
        psk_sp: &PublicSpendKey,
        ssk: &SecretSpendKey,
        challenge: &JubJubScalar,
        license_hash: &str,
        rng: &mut OsRng,
    ) -> Result<Option<SessionCookie>, Error> {
        let pos_license = MoatCoreUtils::get_license_to_use(
            &moat_context.blockchain_access_config,
            ssk,
            license_hash.to_owned(),
        )
        .await?;

        match pos_license {
            Some((pos, license)) => {
                println!(
                    "using license: {}",
                    MoatCoreUtils::to_hash_hex(&license)
                );

                let (_tx_id, session_cookie) =
                    MoatCoreUtils::prove_and_send_use_license(
                        &moat_context.blockchain_access_config,
                        &moat_context.wallet_path,
                        &moat_context.wallet_password,
                        psk_lp,
                        psk_sp,
                        ssk,
                        challenge,
                        &license,
                        pos,
                        moat_context.gas_limit,
                        moat_context.gas_price,
                        &mut None,
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
    blockchain_access_config: BlockchainAccessConfig,
    wallet_path: WalletPath,
    wallet_password: Password,
    gas_limit: u64,
    gas_price: u64,
}

impl MoatContext {
    /// Create a new Moat Context given the required configurations
    pub fn create(
        config_path: &String,
        wallet_path: &String,
        wallet_password: &String,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<Self, Error> {
        let wallet_path = WalletPath::from(Path::new(&wallet_path));
        let wallet_password = Pwd(wallet_password.to_string());

        let blockchain_access_config =
            BlockchainAccessConfig::load_path(config_path)?;

        Ok(Self {
            blockchain_access_config,
            wallet_path,
            wallet_password,
            gas_limit,
            gas_price,
        })
    }
}
