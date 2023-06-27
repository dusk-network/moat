// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{archived_root, Archive, Deserialize, Infallible};

use crate::error::Error;
use crate::retrieval_types::TxJson;
use crate::TxRetriever;
use base64::{engine::general_purpose, Engine as _};
use gql_client::Client;

pub struct PayloadRetriever;

impl PayloadRetriever {
    pub async fn retrieve_tx_payload<P, S>(
        txid: S,
        client: &Client,
    ) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>,
        S: AsRef<str>,
    {
        let tx = TxRetriever::retrieve_tx(txid.as_ref(), client).await?;
        let tx_json: TxJson = serde_json::from_str(tx.json.as_str())?;
        Self::extract_tx_payload(&tx_json)
    }

    pub fn extract_tx_payload<P>(tx_json: &TxJson) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>,
    {
        let payload_base64 = &tx_json.call.CallData;
        let payload_ser = general_purpose::STANDARD.decode(payload_base64)?;

        let payload = unsafe { archived_root::<P>(payload_ser.as_slice()) };
        let p: P = payload.deserialize(&mut Infallible).expect("Infallible");
        Ok(p)
    }
}
