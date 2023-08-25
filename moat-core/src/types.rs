// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::JsonLoader;

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
    pub txid: String,
    pub contractinfo: ContractInfo,
    pub json: String,
}

impl JsonLoader for Tx {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Tx2 {
    pub id: String,
    #[serde(alias = "callData", default)]
    pub call_data: Option<CallInfoJson2>,
    pub raw: String,
}

impl JsonLoader for Tx2 {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Transactions {
    pub transactions: Vec<Tx>,
}

impl JsonLoader for Transactions {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Transactions2 {
    pub transactions: Vec<Tx2>,
}

impl JsonLoader for Transactions2 {}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[allow(non_snake_case)]
pub struct CallInfoJson {
    pub ContractID: String,
    pub FnName: String,
    pub CallData: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct CallInfoJson2 {
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
pub struct TxJson2 {
    pub anchor: String,
    pub call: CallInfoJson2,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Header {
    pub height: u64,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct BlockWithTxs {
    pub header: Header,
    pub transactions: Vec<Tx>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Blocks {
    pub blocks: Vec<BlockWithTxs>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Block {
    pub header: Header,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct QueryResult {
    pub blocks: Vec<Block>,
    pub transactions: Vec<Tx>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct QueryResult2 {
    #[serde(alias = "blockTxs", default)]
    pub block_txs: Vec<Tx2>,
}

// {"block":{"header":{"height":77065}}}
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct QueryResult3 {
    pub block: Block,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTx {
    pub txerror: Option<String>,
    #[serde(alias = "gasSpent", default)]
    pub gas_spent: f64,
    pub tx: Tx2,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTxResponse {
    pub tx: Option<SpentTx>,
}
