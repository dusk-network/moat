// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::{
    ContractInquirerWs, LicenseSession, LicenseSessionId,
    GET_LICENSES_METHOD_NAME, GET_SESSION_METHOD_NAME, LICENSE_CONTRACT_ID,
};
use std::ops::Range;

pub struct CitadelInquirerWs {}

impl CitadelInquirerWs {
    pub async fn get_licenses(
        url: impl AsRef<str>,
        id: Option<i32>,
        block_heights: Range<u64>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        ContractInquirerWs::query_contract(
            url,
            id,
            block_heights,
            LICENSE_CONTRACT_ID,
            GET_LICENSES_METHOD_NAME,
        )
        .await
    }

    pub async fn get_session(
        url: impl AsRef<str>,
        id: Option<i32>,
        session_id: LicenseSessionId,
    ) -> Result<Option<LicenseSession>, Error> {
        ContractInquirerWs::query_contract(
            url,
            id,
            session_id,
            LICENSE_CONTRACT_ID,
            GET_SESSION_METHOD_NAME,
        )
        .await
    }
}
