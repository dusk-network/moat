// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::{ContractInquirer, LicenseSession, LicenseSessionId, DEPTH, ARITY};
use dusk_wallet::RuskHttpClient;
use phoenix_core::transaction::ModuleId;
use poseidon_merkle::Opening;
use std::ops::Range;

// todo: refactor such consts to some common location
const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x03;
    bytes
};

const GET_LICENSES_METHOD_NAME: &str = "get_licenses";
const GET_MERKLE_OPENING_METHOD_NAME: &str = "get_merkle_opening";
const GET_SESSION_METHOD_NAME: &str = "get_session";

pub struct CitadelInquirer {}

impl CitadelInquirer {
    pub async fn get_licenses(
        client: &RuskHttpClient,
        block_heights: Range<u64>,
    ) -> Result<Vec<(u64, Vec<u8>)>, Error> {
        ContractInquirer::query_contract(
            client,
            block_heights,
            LICENSE_CONTRACT_ID,
            GET_LICENSES_METHOD_NAME,
        )
        .await
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
}
