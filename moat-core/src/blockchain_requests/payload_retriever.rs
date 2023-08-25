// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use dusk_wallet::RuskHttpClient;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Infallible};

use crate::blockchain_requests::PayloadExtractor;
use crate::error::Error;
use crate::TxRetriever;
use gql_client::Client;

pub struct PayloadRetriever;

impl PayloadRetriever {
    /// Retrieves payload of a transaction with a given tx id
    pub async fn retrieve_payload<P, S>(
        txid: S,
        client: &RuskHttpClient,
    ) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
        S: AsRef<str>,
    {
        println!("1 about to retrieve tx {}", txid.as_ref());
        let r = TxRetriever::retrieve_tx(txid.as_ref(), client).await;
        println!("3 r.is_err()={}", r.is_err());
        let tx = r?;
        println!("3 retrieved tx");
        let r = PayloadExtractor::payload_from_tx(&tx);
        println!("4 returning from retrieve_payload with r.is_err()={:?}", r.is_err());
        r
    }
}
