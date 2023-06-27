// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::{PayloadExtractor, Transactions, TxRetriever};
use gql_client::Client;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

pub struct RequestScanner;

impl RequestScanner {
    pub fn scan_transactions(txs: Transactions) -> Vec<Request> {
        let mut requests = Vec::new();
        for tx in &txs.transactions {
            if let Ok(request) = PayloadExtractor::extract_payload_from_tx::<Request>(tx) {
                requests.push(request)
            }
        }
        requests
    }

    pub async fn scan_last_blocks(
        last_n_blocks: u32,
        cfg: &BlockchainAccessConfig,
    ) -> Result<Vec<Request>, Error> {
        let client = Client::new(cfg.graphql_address.clone());
        let txs = TxRetriever::retrieve_txs_from_last_n_blocks(
            &client,
            last_n_blocks,
        )
        .await?;
        let requests = RequestScanner::scan_transactions(txs);
        Ok(requests)
    }

    /// Returns found requests and current top block height
    pub async fn scan_block_range(
        height_beg: u64,
        height_end: u64,
        cfg: &BlockchainAccessConfig,
    ) -> Result<(Vec<Request>, u64), Error> {
        let client = Client::new(cfg.graphql_address.clone());
        let (txs, top) = TxRetriever::retrieve_txs_from_block_range(
            &client, height_beg, height_end,
        )
        .await?;
        let requests = RequestScanner::scan_transactions(txs);
        Ok((requests, top))
    }
}
