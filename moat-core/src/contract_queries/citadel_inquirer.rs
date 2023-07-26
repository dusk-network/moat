// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::ops::Range;
use phoenix_core::transaction::ModuleId;
use crate::ContractInquirer;
use crate::error::Error;

// todo: refactor such consts to some common location
const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x03;
    bytes
};

const GET_LICENSES_METHOD_NAME: &str = "get_licenses";

pub struct CitadelInquirer {}

impl CitadelInquirer {
    pub async fn get_licenses(url: impl AsRef<str>, id: Option<i32>, block_heights: Range<u64>) -> Result<Vec<Vec<u8>>, Error> {
        ContractInquirer::query_contract(
            url,
            id,
            block_heights,
            LICENSE_CONTRACT_ID,
            GET_LICENSES_METHOD_NAME
        ).await
    }
}
