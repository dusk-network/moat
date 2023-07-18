// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_jubjub::JubJubScalar;
use dusk_pki::SecretSpendKey;
use dusk_wallet::WalletPath;
use moat_core::{Error, PayloadSender};
use rand::{CryptoRng, RngCore};
use wallet_accessor::{BlockchainAccessConfig, Password};
use zk_citadel::license::{License, Request};

pub struct LicenseIssuer {
    config: BlockchainAccessConfig,
    wallet_path: WalletPath,
    password: Password,
    gas_limit: u64,
    gas_price: u64,
}

// todo: explain how are user attributes going to be passed in here from the
// user
const USER_ATTRIBUTES: u64 = 0x9b308734u64;

#[allow(dead_code)]
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
    ) -> Result<(), Error> {
        let attr = JubJubScalar::from(USER_ATTRIBUTES);
        let license = License::new(&attr, ssk_lp, request, rng);
        PayloadSender::send(
            license,
            &self.config,
            &self.wallet_path,
            &self.password,
            self.gas_limit,
            self.gas_price,
        )
        .await?;

        Ok(())
    }
}
