// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use base64::{Engine as _, engine::general_purpose};
use crate::error::Error;
use gql_client::Client;
use wallet_accessor::BlockchainAccessConfig;
use crate::request_json::RequestBody;
use crate::RequestJson;

pub struct RequestRetriever;

#[derive(Debug, serde::Deserialize)]
struct ContractInfo {
    pub method: String,
    pub contract: String,
}

#[derive(Debug, serde::Deserialize)]
struct Tx {
    pub txid: String,
    pub contractinfo: ContractInfo,
    pub json: String,
}

#[derive(Debug, serde::Deserialize)]
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

impl RequestRetriever {
    pub async fn retrieve(cfg: &BlockchainAccessConfig) -> Result<(), Error>{
        let client = Client::new(cfg.graphql_address.clone());

        let txid= "3eb9e81322f484b4367db41dde033f1b025436a05fc86344009f6b5097cd300d";

        let query =
            "{transactions(txid:\"####\"){ txid, contractinfo{method, contract}, json}}".replace("####", txid);

        let response = client.query::<Transactions>(&query).await.expect("todo:");// todo: remove expect and replace with ?

        let tx_json: TxJson = serde_json::from_str(response.as_ref().unwrap().transactions.get(0).unwrap().json.as_str()).expect("json conversion should work");

        let request_base64 = tx_json.call.CallData.clone();
        let request_rkyv = general_purpose::STANDARD.decode(request_base64.clone()).unwrap();
        let request_body: RequestBody = RequestJson::from_request_rkyv(request_rkyv.clone());

        println!("resp={:?}", response);
        println!("tx_json={:?}", tx_json);
        println!("request_base64={:?}", request_base64);
        println!("request_rkyv={}", hex::encode(request_rkyv));
        println!("request_body={:?}", request_body);

        Ok(())
    }
}
