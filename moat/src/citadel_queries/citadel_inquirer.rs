// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::Error::InvalidQueryResponse;
use crate::{BlockInPlace, StreamAux};
use crate::{
    ContractInquirer, LicenseSession, LicenseSessionId, ARITY, DEPTH,
    GET_INFO_METHOD_NAME, GET_LICENSES_METHOD_NAME,
    GET_MERKLE_OPENING_METHOD_NAME, GET_SESSION_METHOD_NAME,
    LICENSE_CONTRACT_ID,
};
use bytes::Bytes;
use dusk_pki::SecretSpendKey;
use dusk_wallet::RuskHttpClient;
use poseidon_merkle::Opening;
use rkyv::{check_archived_root, Deserialize, Infallible};
use std::ops::Range;
use zk_citadel::license::License;

pub struct CitadelInquirer {}

impl CitadelInquirer {
    // vector overhead length is needed because get_licenses returns licenses
    // serialized as vector of bytes
    const VEC_OVERHEAD_LEN: usize = 8;
    pub const GET_LICENSES_ITEM_LEN: usize =
        std::mem::size_of::<(u64, License)>() + Self::VEC_OVERHEAD_LEN;

    /// Provides licenses issued within a given block height range
    pub async fn get_licenses(
        client: &RuskHttpClient,
        block_heights: Range<u64>,
    ) -> Result<
        impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>,
        Error,
    > {
        ContractInquirer::query_contract_with_feeder(
            client,
            block_heights,
            LICENSE_CONTRACT_ID,
            GET_LICENSES_METHOD_NAME,
        )
        .wait()
    }

    /// Provides opening for a given position in the merkle tree, or None if not
    /// found.
    pub async fn get_merkle_opening(
        client: &RuskHttpClient,
        pos: u64,
    ) -> Result<Option<Opening<(), DEPTH, ARITY>>, Error> {
        ContractInquirer::query_contract(
            client,
            pos,
            LICENSE_CONTRACT_ID,
            GET_MERKLE_OPENING_METHOD_NAME,
        )
        .await
    }

    /// Provides session with a given session id, or None if not found.
    pub async fn get_session(
        client: &RuskHttpClient,
        session_id: LicenseSessionId,
    ) -> Result<Option<LicenseSession>, Error> {
        ContractInquirer::query_contract(
            client,
            session_id,
            LICENSE_CONTRACT_ID,
            GET_SESSION_METHOD_NAME,
        )
        .await
    }

    /// Provides information about license contract's state.
    pub async fn get_info(
        client: &RuskHttpClient,
    ) -> Result<(u32, u32, u32), Error> {
        ContractInquirer::query_contract(
            client,
            (),
            LICENSE_CONTRACT_ID,
            GET_INFO_METHOD_NAME,
        )
        .await
    }

    /// Deserializes license, panics if deserialization fails.
    fn deserialise_license(v: &Vec<u8>) -> License {
        let response_data = check_archived_root::<License>(v.as_slice())
            .map_err(|_| {
                InvalidQueryResponse("rkyv deserialization error".into())
            })
            .expect("License should deserialize correctly");
        let license: License = response_data
            .deserialize(&mut Infallible)
            .expect("Infallible");
        license
    }

    /// Finds owned license in a stream of licenses.
    /// It searches in a reverse order to return a newest license.
    pub fn find_owned_licenses(
        ssk_user: SecretSpendKey,
        stream: &mut (impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
                  + std::marker::Unpin),
    ) -> Result<Vec<(u64, License)>, Error> {
        const ITEM_LEN: usize = CitadelInquirer::GET_LICENSES_ITEM_LEN;
        let mut pairs = vec![];
        StreamAux::find_items::<(u64, Vec<u8>), ITEM_LEN>(
            |(pos, lic_vec)| {
                let license = Self::deserialise_license(lic_vec);
                if ssk_user.view_key().owns(&license.lsa) {
                    pairs.push((*pos, license));
                };
            },
            stream,
        )?;
        Ok(pairs)
    }

    /// Finds owned license in a stream of licenses.
    /// It searches in a reverse order to return a newest license.
    pub fn find_all_licenses(
        stream: &mut (impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
                  + std::marker::Unpin),
    ) -> Result<Vec<(u64, License)>, Error> {
        const ITEM_LEN: usize = CitadelInquirer::GET_LICENSES_ITEM_LEN;
        let mut pairs = vec![];
        StreamAux::find_items::<(u64, Vec<u8>), ITEM_LEN>(
            |(pos, lic_vec)| {
                let license = Self::deserialise_license(lic_vec);
                pairs.push((*pos, license));
            },
            stream,
        )?;
        Ok(pairs)
    }
}
