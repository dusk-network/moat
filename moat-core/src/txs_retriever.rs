// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::retrieval_types::{Blocks, Transactions};
use gql_client::Client;

pub struct TxsRetriever;

impl TxsRetriever {
    pub async fn retrieve_txs_from_block(
        client: &Client,
        block_height: u64,
    ) -> Result<Transactions, Error> {
        TxsRetriever::retrieve_txs_from_block_range(
            client,
            block_height,
            block_height + 1,
        )
        .await
    }

    // range retrieval seems to have a limit of 10k
    pub async fn retrieve_txs_from_block_range(
        client: &Client,
        height_beg: u64,
        height_end: u64,
    ) -> Result<Transactions, Error> {
        let mut transactions = Transactions::default();
        let range_str = format!("{},{}", height_beg, height_end);
        println!("retrieving {}", range_str);
        let query =
            "{transactions(blocksrange: [####]){txid, contractinfo{method, contract}, json}}".replace("####", range_str.as_str());
        let result = client
            .query::<Transactions>(&query)
            .await
            .map_err(|e| e.into());
        match result {
            e @ Err(_) => return e.map(|_| Transactions::default()),
            Ok(None) => (),
            Ok(Some(txs)) => {
                transactions.transactions.extend(txs.transactions);
            }
        }
        Ok(transactions)
    }

    pub async fn retrieve_txs_from_last_n_blocks(
        client: &Client,
        n: u32,
    ) -> Result<Transactions, Error> {
        let mut transactions = Transactions::default();
        let n_str = format!("{}", n);
        println!("retrieving {}", n_str);
        let query =
            "{blocks(last:9999){ header{height, seed }, transactions{txid, contractinfo{method, contract}, json}}}".replace("9999", n_str.as_str());
        let result = client.query::<Blocks>(&query).await.map_err(|e| e.into());
        match result {
            e @ Err(_) => return e.map(|_| Transactions::default()),
            Ok(None) => (),
            Ok(Some(blocks)) => {
                for block in blocks.blocks {
                    transactions.transactions.extend(block.transactions);
                }
            }
        }
        Ok(transactions)
    }
}
