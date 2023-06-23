// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

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

// todo: duplication
impl Tx {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Tx, Error> {
        let mut content = String::new();
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content)?;
        serde_json::from_str(&content).map_err(|e| e.into())
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Transactions {
    pub transactions: Vec<Tx>,
}

// todo: duplication
impl Transactions {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Transactions, Error> {
        let mut content = String::new();
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content)?;
        serde_json::from_str(&content).map_err(|e| e.into())
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[allow(non_snake_case)]
pub struct CallInfoJson {
    pub ContractID: String,
    pub FnName: String,
    pub CallData: String,
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
