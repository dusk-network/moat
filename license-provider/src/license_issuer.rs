// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::{BlsScalar, JubJubAffine, JubJubScalar};
use dusk_pki::SecretSpendKey;
use dusk_poseidon::sponge;
use dusk_wallet::{RuskHttpClient, WalletPath};
use moat_core::{
    Error, PayloadSender, TxAwaiter, ISSUE_LICENSE_METHOD_NAME,
    LICENSE_CONTRACT_ID, MAX_LICENSE_SIZE,
};
use rand::{CryptoRng, RngCore};
use tracing::trace;
use wallet_accessor::{BlockchainAccessConfig, Password};
use zk_citadel::license::{License, Request};

pub struct LicenseIssuer {
    config: BlockchainAccessConfig,
    wallet_path: WalletPath,
    password: Password,
    gas_limit: u64,
    gas_price: u64,
}

const USER_ATTRIBUTES: u64 = 1 << 17;

impl LicenseIssuer {
    pub fn new(
        config: BlockchainAccessConfig,
        wallet_path: WalletPath,
        password: Password,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            config,
            wallet_path,
            password,
            gas_limit,
            gas_price,
        }
    }

    pub async fn issue_license<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
        request: &Request,
        ssk_lp: &SecretSpendKey,
    ) -> Result<BlsScalar, Error> {
        let attr = JubJubScalar::from(USER_ATTRIBUTES);
        let license = License::new(&attr, ssk_lp, request, rng);
        let license_blob = rkyv::to_bytes::<_, MAX_LICENSE_SIZE>(&license)
            .expect("License should serialize correctly")
            .to_vec();
        let lpk = JubJubAffine::from(license.lsa.pk_r().as_ref());
        let license_hash = sponge::hash(&[lpk.get_x(), lpk.get_y()]);
        let tuple = (license_blob, license_hash);
        trace!(
            "sending issue license with license blob size={}",
            tuple.0.len()
        );
        let tx_id = PayloadSender::send_to_contract_method(
            tuple,
            &self.config,
            &self.wallet_path,
            &self.password,
            self.gas_limit,
            self.gas_price,
            LICENSE_CONTRACT_ID,
            ISSUE_LICENSE_METHOD_NAME,
        )
        .await?;
        let client = RuskHttpClient::new(self.config.rusk_address.clone());
        TxAwaiter::wait_for(&client, tx_id).await?;
        Ok(tx_id)
    }
}
