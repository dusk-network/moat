// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use dusk_bytes::DeserializableSlice;
use dusk_jubjub::{JubJubAffine, JubJubScalar, GENERATOR_EXTENDED};
use dusk_pki::{PublicSpendKey, SecretSpendKey};
use dusk_poseidon::sponge;
use rand::{CryptoRng, RngCore};
use zk_citadel::license::Request;

pub struct RequestCreator;

impl RequestCreator {
    pub fn create<R: RngCore + CryptoRng>(
        ssk_user: &SecretSpendKey,
        psk_lp: &PublicSpendKey,
        rng: &mut R,
    ) -> Result<Request, Error> {
        let psk = ssk_user.public_spend_key();
        let lsa = psk.gen_stealth_address(&JubJubScalar::random(rng));
        let lsk = ssk_user.sk_r(&lsa);
        let k_lic = JubJubAffine::from(
            GENERATOR_EXTENDED
                * sponge::truncated::hash(&[(*lsk.as_ref()).into()]),
        );
        let request = Request::new(psk_lp, &lsa, &k_lic, rng);
        Ok(request)
    }
    pub fn create_from_hex<
        R: RngCore + CryptoRng,
        S: AsRef<str>,
        T: AsRef<str>,
    >(
        ssk_user: S,
        psk_lp: T,
        rng: &mut R,
    ) -> Result<Request, Error> {
        let ssk_user_bytes = hex::decode(ssk_user.as_ref())?;
        let ssk_user = SecretSpendKey::from_slice(ssk_user_bytes.as_slice())?;
        let psk_lp_bytes = hex::decode(psk_lp.as_ref())?;
        let psk_lp = PublicSpendKey::from_slice(psk_lp_bytes.as_slice())?;
        Self::create(&ssk_user, &psk_lp, rng)
    }
}
