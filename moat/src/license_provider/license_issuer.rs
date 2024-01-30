// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::api::{MoatContext, MoatCore};
use crate::{
    Error, PayloadSender, TxAwaiter, ISSUE_LICENSE_METHOD_NAME,
    LICENSE_CONTRACT_ID, MAX_LICENSE_SIZE,
};
use dusk_bls12_381::BlsScalar;
use dusk_jubjub::{JubJubAffine, JubJubScalar};
use dusk_poseidon::sponge;
use dusk_wallet::RuskHttpClient;
use rand::{CryptoRng, RngCore};
use tracing::trace;
use zk_citadel::license::{License, Request};

pub struct LicenseIssuer {}

impl LicenseIssuer {
    /// Issue license for a given request, License Provider SSK, and attribute
    /// data. Returns a serialized license.
    pub async fn issue_license<R: RngCore + CryptoRng>(
        rng: &mut R,
        request: &Request,
        attr_data: &JubJubScalar,
        moat_context: &MoatContext,
    ) -> Result<(BlsScalar, Vec<u8>), Error> {
        let (_psk, ssk) = MoatCore::get_wallet_keypair(moat_context)?;
        let license = License::new(attr_data, &ssk, request, rng);
        let license_blob = rkyv::to_bytes::<_, MAX_LICENSE_SIZE>(&license)
            .expect("Serializing should be infallible")
            .to_vec();
        let lpk = JubJubAffine::from(license.lsa.pk_r().as_ref());
        let license_hash = sponge::hash(&[lpk.get_u(), lpk.get_v()]);
        let tuple = (license_blob.clone(), license_hash);
        trace!(
            "sending issue license with license blob size={}",
            tuple.0.len()
        );
        let tx_id = PayloadSender::execute_contract_method(
            tuple,
            moat_context,
            LICENSE_CONTRACT_ID,
            ISSUE_LICENSE_METHOD_NAME,
        )
        .await?;
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        TxAwaiter::wait_for(&client, tx_id).await?;
        Ok((tx_id, license_blob))
    }
}
