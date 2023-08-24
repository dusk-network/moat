// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::types::*;
use crate::Error::{DuskWalletError, TransactionNotFound};
use crate::{QueryResult, Tx};
use dusk_wallet::{RuskHttpClient, RuskRequest};
use gql_client::Client;
use std::sync::Arc;

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
    ) -> Result<Transactions2, Error> {
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
    ) -> Result<(Transactions2, u64), Error> {
        let mut transactions = Transactions2::default();
        let range_str = format!("{},{}", height_beg, height_end);
        let tx_query = "query { blockTxs(range: [####] ) { id, raw, callData {contractId, fnName, data}}}".replace("####", range_str.as_str());
        let tx_response = gql_query(client, tx_query.as_str()).await?;
        let tx_result = serde_json::from_slice::<QueryResult2>(&tx_response)?;
        let top_block_query =
            "query { block(height: -1) { header { height} }}".to_string();
        let top_block_response =
            gql_query(client, top_block_query.as_str()).await?;
        let top_block_result =
            serde_json::from_slice::<QueryResult3>(&top_block_response)?;

        transactions.transactions.extend(tx_result.block_txs);
        Ok((transactions, top_block_result.block.header.height))
    }

    pub async fn txs_from_last_n_blocks(
        client: &RuskHttpClient,
        n: usize,
    ) -> Result<Transactions2, Error> {
        let mut transactions = Transactions2::default();
        let n_str = format!("{}", n);
        let tx_query = "query { blockTxs(last:####) { id, raw, callData {contractId, fnName, data}}}".replace("####", n_str.as_str());
        let tx_response = gql_query(client, tx_query.as_str()).await?;
        let tx_result = serde_json::from_slice::<QueryResult2>(&tx_response)?;
        transactions.transactions.extend(tx_result.block_txs);
        Ok(transactions)
    }

    pub async fn retrieve_tx<S>(txid: S, client: &Client) -> Result<Tx2, Error>
    where
        S: AsRef<str>,
    {
        let query =
            "{transactions(txid:\"####\"){ txid, contractinfo{method, contract}, json}}".replace("####", txid.as_ref());
        let response = client.query::<Transactions2>(&query).await?;
        match response {
            Some(Transactions2 {
                transactions: mut txs,
            }) if !txs.is_empty() => Ok(txs.swap_remove(0)),
            _ => Err(TransactionNotFound),
        }
    }
}
