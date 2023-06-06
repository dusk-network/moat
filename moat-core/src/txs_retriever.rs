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

    pub async fn retrieve_txs_from_block_range(
        client: &Client,
        height_beg: u64,
        height_end_excl: u64,
    ) -> Result<Transactions, Error> {
        let mut transactions = Transactions::default();
        for height in height_beg..height_end_excl {
            let single_block_query = height_end_excl == height_beg + 1;
            let height_str = format!("{}", height);
            println!("retrieving {}", height_str);
            let query =
                "{blocks(height:9999){ header{height, seed }, transactions{txid, contractinfo{method, contract}, json}}}".replace("9999", height_str.as_str());
            let result =
                client.query::<Blocks>(&query).await.map_err(|e| e.into());
            match result {
                e @ Err(_) if single_block_query => {
                    return e.map(|_| Transactions::default())
                }
                Err(_) | Ok(None) => (),
                Ok(Some(blocks)) => {
                    for block in blocks.blocks {
                        transactions.transactions.extend(block.transactions);
                    }
                }
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
