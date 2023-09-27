// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::JsonLoader;
use phoenix_core::transaction::ModuleId;

pub const STAKE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x02;
    bytes
};
pub const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x03;
    bytes
};

// license contract method names
pub const NOOP_METHOD_NAME: &str = "noop";
pub const ISSUE_LICENSE_METHOD_NAME: &str = "issue_license";
pub const USE_LICENSE_METHOD_NAME: &str = "use_license";
pub const GET_LICENSES_METHOD_NAME: &str = "get_licenses";
pub const GET_MERKLE_OPENING_METHOD_NAME: &str = "get_merkle_opening";
pub const GET_SESSION_METHOD_NAME: &str = "get_session";
pub const GET_INFO_METHOD_NAME: &str = "get_info";

// stake contract method names
pub const ADD_OWNER_METHOD_NAME: &str = "add_owner";

pub const MAX_CALL_SIZE: usize = 65536;
pub const MAX_REQUEST_SIZE: usize = 8192;
pub const MAX_LICENSE_SIZE: usize = 16384;
pub const MAX_RESPONSE_SIZE: usize = 65536;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct RequestJson {
    pub user_ssk: String,
    pub provider_psk: String,
}

impl JsonLoader for RequestJson {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ContractInfo {
    pub method: String,
    pub contract: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Tx {
    pub id: String,
    #[serde(alias = "callData", default)]
    pub call_data: Option<CallInfoJson>,
    pub raw: String,
}

impl JsonLoader for Tx {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Transactions {
    pub transactions: Vec<Tx>,
}

impl JsonLoader for Transactions {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct CallInfoJson {
    #[serde(alias = "contractId", default)]
    pub contract_id: String,
    #[serde(alias = "fnName", default)]
    pub fn_name: String,
    pub data: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TxJson {
    pub anchor: String,
    pub call: CallInfoJson,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Header {
    pub height: u64,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Block {
    pub header: Header,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct QueryResult {
    #[serde(alias = "blockTxs", default)]
    pub block_txs: Vec<Tx>,
}

// {"block":{"header":{"height":77065}}}
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct QueryResult2 {
    pub block: Block,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTx {
    pub err: Option<String>,
    #[serde(alias = "gasSpent", default)]
    pub gas_spent: u64,
    #[serde(alias = "blockHeight", default)]
    pub block_height: u64,
    pub tx: Tx,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTx2 {
    pub err: Option<String>,
    #[serde(alias = "gasSpent", default)]
    pub gas_spent: u64,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTxResponse {
    pub tx: Option<SpentTx>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTxResponse2 {
    pub tx: Option<SpentTx2>,
}

#[derive(Debug)]
pub enum TxStatus {
    Ok,
    NotFound,
    Error(String),
}
