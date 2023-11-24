// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::JsonLoader;
use phoenix_core::transaction::ModuleId;

pub const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x03;
    bytes
};

pub const MAX_REQUEST_SIZE: usize = 8192;
pub const MAX_LICENSE_SIZE: usize = 16384;

// license contract method names
pub const NOOP_METHOD_NAME: &str = "request_license";
pub const ISSUE_LICENSE_METHOD_NAME: &str = "issue_license";
pub const USE_LICENSE_METHOD_NAME: &str = "use_license";
pub const GET_LICENSES_METHOD_NAME: &str = "get_licenses";
pub const GET_MERKLE_OPENING_METHOD_NAME: &str = "get_merkle_opening";
pub const GET_SESSION_METHOD_NAME: &str = "get_session";
pub const GET_INFO_METHOD_NAME: &str = "get_info";

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct RequestJson {
    pub user_ssk: String,
    pub provider_psk: String,
}

impl JsonLoader for RequestJson {}
