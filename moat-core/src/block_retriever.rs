// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gql_client::Client;
use crate::error::Error;
use crate::retrieval_types::{Blocks, Transactions};

pub struct RequestRetriever;

impl RequestRetriever {
    pub async fn retrieve_txs_from_block(
        client: &Client,
        block_height: u64,
    ) -> Result<Transactions, Error> {
        let block_height_str = format!("{}", block_height);

        let mut transactions = Transactions::default();

        let query =
            "{blocks(height:9999){ header{height, seed }, transactions{txid, contractinfo{method, contract}, json}}}".replace("9999", block_height_str.as_str());

        let result = client.query::<Blocks>(&query).await.map_err(|e|e.into());

        match result {
            e@Err(_) => e.map(|_|Transactions::default()),
            Ok(None) => Ok(Transactions::default()),
            Ok(Some(blocks)) => {
                for block in blocks.blocks {
                    transactions.transactions.extend(block.transactions);
                }
                Ok(transactions)
            }
        }
    }
}
