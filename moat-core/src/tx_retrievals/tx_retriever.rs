// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;
use dusk_wallet::{RuskHttpClient, RuskRequest};
use crate::error::Error;
use crate::types::*;
use crate::Error::{DuskWalletError, TransactionNotFound};
use crate::{QueryResult, Tx};
use gql_client::Client;

pub struct TxRetriever;


async fn gql_query(
    client: &RuskHttpClient,
    query: &str,
) -> Result<Vec<u8>, dusk_wallet::Error> {
    let request = RuskRequest::new("gql", query.as_bytes().to_vec());
    client.call(2, "Chain", &request).await
}


impl TxRetriever {
    pub async fn txs_from_block(
        client: &RuskHttpClient,
        block_height: u64,
    ) -> Result<Transactions, Error> {
        TxRetriever::txs_from_block_range(
            client,
            block_height,
            block_height + 1,
        )
        .await
        .map(|(txs, _)| txs)
    }

    // range retrieval seems to have a limit of 10k
    /// returns transactions in a range and the current top block
    pub async fn txs_from_block_range(
        client: &RuskHttpClient,
        height_beg: u64,
        height_end: u64,
    ) -> Result<(Transactions, u64), Error> {
        let mut transactions = Transactions::default();
        let mut top_block: u64 = 0;
        let range_str = format!("{},{}", height_beg, height_end);
        // let query =
        //     "{blocks(height:-1){header{height}}, transactions(blocksrange: [####]){txid, contractinfo{method, contract}, json}}".replace("####", range_str.as_str());
        let query = "query { blockTxs(range: [288,22271] ) { id, raw, callData {fnName}}}".to_string(); //.replace("####", range_str.as_str());
        // let result = client
        //     .query::<QueryResult>(&query)
        //     .await
        //     .map_err(|e| e.into());
        let response = gql_query(client, query.as_str()).await.map_err(|e| DuskWalletError(Arc::new(e)))?; // todo: move error conv to Error
        let result = serde_json::from_slice::<QueryResult2>(&response).map_err(|e| e.into());

        match result {
            e @ Err(_) => {
                return e.map(|_| (Transactions::default(), top_block));
            }
            // Ok(None) => (),
            // Ok(Some(query_result)) => {
            Ok(query_result) => {
                println!("{:?}", query_result);
                // transactions.transactions.extend(query_result.transactions);
                // top_block = query_result
                //     .blocks
                //     .get(0)
                //     .map(|a| a.header.height)
                //     .unwrap_or(0u64);
            }
        }
        Ok((transactions, top_block))
    }

    pub async fn txs_from_last_n_blocks(
        client: &Client,
        n: usize,
    ) -> Result<Transactions, Error> {
        let mut transactions = Transactions::default();
        let n_str = format!("{}", n);
        let query =
            "{blocks(last:9999){ header{height}, transactions{txid, contractinfo{method, contract}, json}}}".replace("9999", n_str.as_str());
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

    pub async fn retrieve_tx<S>(txid: S, client: &Client) -> Result<Tx, Error>
    where
        S: AsRef<str>,
    {
        let query =
            "{transactions(txid:\"####\"){ txid, contractinfo{method, contract}, json}}".replace("####", txid.as_ref());

        let response = client.query::<Transactions>(&query).await?;
        match response {
            Some(Transactions {
                transactions: mut txs,
            }) if !txs.is_empty() => Ok(txs.swap_remove(0)),
            _ => Err(TransactionNotFound),
        }
    }
}
