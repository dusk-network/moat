// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Infallible};

use crate::error::Error;
use crate::retrieval_types::TxJson;
use crate::{PayloadExtractor, TxRetriever};
use gql_client::Client;

pub struct PayloadRetriever;

impl PayloadRetriever {
    /// Retrieves payload of a transaction with a given tx id
    pub async fn retrieve_payload<P, S>(
        txid: S,
        client: &Client,
    ) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
        S: AsRef<str>,
    {
        let tx = TxRetriever::retrieve_tx(txid.as_ref(), client).await?;
        let tx_json: TxJson = serde_json::from_str(tx.json.as_str())?;
        PayloadExtractor::payload_from_tx_json(&tx_json)
    }
}
