// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::api::MoatContext;
use dusk_pki::PublicSpendKey;
use rand::rngs::OsRng;

use zk_citadel::license::{License, SessionCookie};

use dusk_jubjub::JubJubScalar;

use crate::{BcInquirer, CitadelInquirer, Error, LicenseUser, TxAwaiter};
use dusk_wallet::RuskHttpClient;
use rkyv::ser::serializers::AllocSerializer;
use sha3::{Digest, Sha3_256};

use dusk_bls12_381::BlsScalar;

pub struct MoatCoreUtils {}

const MAX_OBJECT_SIZE: usize = 16384;

impl MoatCoreUtils {
    pub fn to_hash_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<MAX_OBJECT_SIZE>>,
    {
        let blob = rkyv::to_bytes::<_, MAX_OBJECT_SIZE>(object)
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

    pub fn to_blob_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<MAX_OBJECT_SIZE>>,
    {
        let blob = Self::to_blob(object);
        hex::encode(blob)
    }

    pub fn to_blob<T>(object: &T) -> Vec<u8>
    where
        T: rkyv::Serialize<AllocSerializer<MAX_OBJECT_SIZE>>,
    {
        rkyv::to_bytes::<_, MAX_OBJECT_SIZE>(object)
            .expect("Serializing should be infallible")
            .to_vec()
    }

    pub async fn get_license_to_use(
        moat_context: &MoatContext,
        license_hash: String,
    ) -> Result<Option<(u64, License)>, Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );
        let end_height = BcInquirer::block_height(&client).await?;
        let block_heights = 0..(end_height + 1);

        let mut licenses_stream =
            CitadelInquirer::get_licenses(&client, block_heights).await?;

        let pairs = CitadelInquirer::find_owned_licenses(
            moat_context,
            &mut licenses_stream,
        )?;
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
        moat_context: &MoatContext,
        psk_lp: &PublicSpendKey,
        psk_sp: &PublicSpendKey,
        challenge: &JubJubScalar,
        license: &License,
        pos: u64,
        rng: &mut OsRng,
    ) -> Result<(BlsScalar, SessionCookie), Error> {
        let client = RuskHttpClient::new(
            moat_context.blockchain_access_config.rusk_address.clone(),
        );

        let opening = CitadelInquirer::get_merkle_opening(&client, pos)
            .await?
            .expect("Opening obtained successfully");

        let (tx_id, session_cookie) = LicenseUser::prove_and_use_license(
            moat_context,
            psk_lp,
            psk_sp,
            license,
            opening,
            rng,
            challenge,
        )
        .await?;
        TxAwaiter::wait_for(&client, tx_id).await?;
        Ok((tx_id, session_cookie))
    }
}
