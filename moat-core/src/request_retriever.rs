// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use rkyv::{
    archived_root, Archive, Archived, Deserialize, Infallible, Serialize,
};

use crate::error::Error;
use base64::{engine::general_purpose, Engine as _};
use gql_client::Client;
use wallet_accessor::BlockchainAccessConfig;

pub struct RequestRetriever;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct ContractInfo {
    pub method: String,
    pub contract: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Tx {
    pub txid: String,
    pub contractinfo: ContractInfo,
    pub json: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Transactions {
    pub transactions: Vec<Tx>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct CallInfoJson {
    pub ContractID: String,
    pub FnName: String,
    pub CallData: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct TxJson {
    pub anchor: String,
    pub call: CallInfoJson,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Header {
    pub height: u64,
    pub seed: String,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Block {
    pub header: Header,
    pub transactions: Transactions,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Blocks {
    pub blocks: Vec<Block>,
}

impl RequestRetriever {
    pub async fn retrieve_tx_payload<P, S>(
        txid: S,
        cfg: &BlockchainAccessConfig,
    ) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>,
        S: AsRef<str>,
    {
        let client = Client::new(cfg.graphql_address.clone());

        let query =
            "{transactions(txid:\"####\"){ txid, contractinfo{method, contract}, json}}".replace("####", txid.as_ref());

        let response =
            client.query::<Transactions>(&query).await.expect("todo:"); // todo: remove expect and replace with ?

        let tx_json: TxJson = serde_json::from_str(
            response
                .as_ref()
                .unwrap()
                .transactions
                .get(0)
                .unwrap()
                .json
                .as_str(),
        )
        .expect("json conversion should work");

        let payload_base64 = tx_json.call.CallData.clone();
        let payload_ser = general_purpose::STANDARD
            .decode(payload_base64.clone())
            .unwrap();

        let payload = unsafe { archived_root::<P>(payload_ser.as_slice()) };
        let p: P = payload.deserialize(&mut Infallible).expect("Infallible");
        Ok(p)
    }

    pub async fn retrieve_block(
        cfg: &BlockchainAccessConfig,
    ) -> Result<(), Error> {
        let client = Client::new(cfg.graphql_address.clone());

        let block_height = "97117";

        // todo: this will fail if there are no transactions in a given block,
        // so first we need to make sure that a block has transactions
        let query =
            "{blocks(height:9999){ header{height, seed }, transactions{txid, json}}}".replace("9999", block_height);

        let response = client.query::<Blocks>(&query).await.expect("todo:"); // todo: remove expect and replace with ?

        println!("resp={:?}", response);
        println!("blocks={:?}", response.unwrap());

        Ok(())
    }
}
