// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::bc_types::*;
use crate::error::Error;
use crate::BcInquirer;
use crate::Error::TransactionNotFound;
use dusk_wallet::RuskHttpClient;

pub struct TxInquirer;

impl TxInquirer {
    pub async fn txs_from_block(
        client: &RuskHttpClient,
        block_height: u64,
    ) -> Result<Transactions, Error> {
        TxInquirer::txs_from_block_range(client, block_height, block_height + 1)
            .await
            .map(|(txs, _)| txs)
    }

    /// returns transactions in a range and the current top block
    pub async fn txs_from_block_range(
        client: &RuskHttpClient,
        height_beg: u64,
        height_end: u64,
    ) -> Result<(Transactions, u64), Error> {
        let mut transactions = Transactions::default();
        let range_str = format!("{},{}", height_beg, height_end);
        let tx_query = "query { blockTxs(range: [####] ) { tx { id, raw, callData {contractId, fnName, data} } } }".replace("####", range_str.as_str());
        let tx_response =
            BcInquirer::gql_query(client, tx_query.as_str()).await?;
        let tx_result = serde_json::from_slice::<QueryResult>(&tx_response)?;
        transactions
            .transactions
            .extend(tx_result.block_txs.into_iter().map(|t| t.tx));
        let height = BcInquirer::block_height(client).await?;
        Ok((transactions, height))
    }

    pub async fn txs_from_last_n_blocks(
        client: &RuskHttpClient,
        n: usize,
    ) -> Result<Transactions, Error> {
        let mut transactions = Transactions::default();
        let n_str = format!("{}", n);
        let tx_query = "query { blockTxs(last:####) { tx { id, raw, callData {contractId, fnName, data} } } }".replace("####", n_str.as_str());
        let tx_response =
            BcInquirer::gql_query(client, tx_query.as_str()).await?;
        let tx_result = serde_json::from_slice::<QueryResult>(&tx_response)?;
        transactions
            .transactions
            .extend(tx_result.block_txs.into_iter().map(|t| t.tx));
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
