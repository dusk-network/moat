// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::types::*;
use crate::Error::TransactionNotFound;
use crate::{BcInquirer, QueryResult};
use dusk_wallet::RuskHttpClient;

pub struct TxRetriever;

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
        let range_str = format!("{},{}", height_beg, height_end);
        let tx_query = "query { blockTxs(range: [####] ) { id, raw, tx { callData {contractId, fnName, data} } } }".replace("####", range_str.as_str());
        let tx_response =
            BcInquirer::gql_query(client, tx_query.as_str()).await?;
        let tx_result = serde_json::from_slice::<QueryResult>(&tx_response)?;
        let top_block_query =
            "query { block(height: -1) { header { height} }}".to_string();
        let top_block_response =
            BcInquirer::gql_query(client, top_block_query.as_str()).await?;
        let top_block_result =
            serde_json::from_slice::<QueryResult2>(&top_block_response)?;

        transactions.transactions.extend(tx_result.block_txs);
        Ok((transactions, top_block_result.block.header.height))
    }

    pub async fn txs_from_last_n_blocks(
        client: &RuskHttpClient,
        n: usize,
    ) -> Result<Transactions, Error> {
        let mut transactions = Transactions::default();
        let n_str = format!("{}", n);
        let tx_query = "query { blockTxs(last:####) { id, raw, tx { callData {contractId, fnName, data} } } }".replace("####", n_str.as_str());
        let tx_response =
            BcInquirer::gql_query(client, tx_query.as_str()).await?;
        let tx_result = serde_json::from_slice::<QueryResult>(&tx_response)?;
        transactions.transactions.extend(tx_result.block_txs);
        Ok(transactions)
    }

    pub async fn retrieve_tx<S>(
        txid: S,
        client: &RuskHttpClient,
    ) -> Result<(Tx, u64), Error>
    where
        S: AsRef<str>,
    {
        let query = "query { tx(hash:\"####\") { tx {id, raw, callData {contractId, fnName, data}}, blockHeight }}".replace("####", txid.as_ref());
        let response = BcInquirer::gql_query(client, query.as_str()).await?;
        let result = serde_json::from_slice::<SpentTxResponse>(&response)?;
        result
            .tx
            .map(|spent_tx| (spent_tx.tx, spent_tx.block_height))
            .ok_or(TransactionNotFound)
    }
}
