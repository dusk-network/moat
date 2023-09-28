// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::contract_queries::block::Block;
use crate::error::Error;
use crate::{
    ContractInquirer, LicenseSession, LicenseSessionId, ARITY, DEPTH,
    GET_INFO_METHOD_NAME, GET_LICENSES_METHOD_NAME,
    GET_MERKLE_OPENING_METHOD_NAME, GET_SESSION_METHOD_NAME,
    LICENSE_CONTRACT_ID,
};
use bytes::Bytes;
use dusk_wallet::RuskHttpClient;
use poseidon_merkle::Opening;
use std::ops::Range;
use zk_citadel::license::License;

pub struct CitadelInquirer {}

impl CitadelInquirer {
    // vector overhead length is needed as get_licenses returns licenses
    // serialized as vector of bytes
    const VEC_OVERHEAD_LEN: usize = 8;
    pub const GET_LICENSES_ITEM_LEN: usize =
        std::mem::size_of::<(u64, License)>() + Self::VEC_OVERHEAD_LEN;

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
}
