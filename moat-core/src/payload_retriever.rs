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
use crate::retrieval_types::{Transactions, TxJson};
use base64::{engine::general_purpose, Engine as _};
use gql_client::Client;
use wallet_accessor::BlockchainAccessConfig;

pub struct PayloadRetriever;

impl PayloadRetriever {
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

        let response = client.query::<Transactions>(&query).await?;

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
}
