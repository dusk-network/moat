// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::blockchain_payloads::PayloadExtractor;
use crate::error::Error;
use crate::{Transactions, TxInquirer};
use dusk_bls12_381::BlsScalar;
use dusk_wallet::RuskHttpClient;
use phoenix_core::Transaction;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

pub struct RequestScanner;

impl RequestScanner {
    /// Returns requests found in the given collection of transactions
    pub fn scan_transactions(txs: Transactions) -> Vec<Request> {
        let mut requests = Vec::new();
        for tx in &txs.transactions {
            if let Ok(request) =
                PayloadExtractor::payload_from_tx::<Request>(tx)
            {
                requests.push(request)
            }
        }
        requests
    }

    /// Returns requests related to notes from a given list of note hashes.
    pub fn scan_transactions_related_to_notes(
        txs: Transactions,
        note_hashes: &[BlsScalar],
    ) -> (usize, Vec<Request>) {
        let mut requests = Vec::new();
        let mut total_count = 0usize;
        for tx in &txs.transactions {
            if let Ok(request) =
                PayloadExtractor::payload_from_tx::<Request>(tx)
            {
                total_count += 1;
                let tx_raw = hex::decode(&tx.raw)
                    .expect("Decoding raw transaction should succeed");
                let ph_tx = Transaction::from_slice(&tx_raw)
                    .expect("Transaction creation from slice should succeed");
                for note_hash in note_hashes.iter() {
                    if ph_tx
                        .nullifiers()
                        .iter()
                        .any(|&nullifier| nullifier.eq(note_hash))
                    {
                        requests.push(request);
                        break;
                    }
                }
            }
        }
        (total_count, requests)
    }

    /// Returns collection of requests found withing n last blocks
    pub async fn scan_last_blocks(
        last_n_blocks: usize,
        cfg: &BlockchainAccessConfig,
    ) -> Result<Vec<Request>, Error> {
        let client = RuskHttpClient::new(cfg.rusk_address.clone());
        let txs =
            TxInquirer::txs_from_last_n_blocks(&client, last_n_blocks).await?;
        let requests = RequestScanner::scan_transactions(txs);
        Ok(requests)
    }

    /// Returns collection with found requests and the current top block-height
    pub async fn scan_block_range(
        height_beg: u64,
        height_end: u64,
        cfg: &BlockchainAccessConfig,
    ) -> Result<(Vec<Request>, u64), Error> {
        println!("got here 002: {} len={}", cfg.rusk_address.clone(), cfg.rusk_address.len());
        let client = RuskHttpClient::new(cfg.rusk_address.clone());
        let (txs, top) =
            TxInquirer::txs_from_block_range(&client, height_beg, height_end)
                .await?;
        let requests = RequestScanner::scan_transactions(txs);
        Ok((requests, top))
    }

    /// Scans requests related to notes from a given list of note hashes.
    pub async fn scan_related_to_notes_in_block_range(
        height_beg: u64,
        height_end: u64,
        cfg: &BlockchainAccessConfig,
        note_hashes: &[BlsScalar],
    ) -> Result<(Vec<Request>, u64, usize), Error> {
        let client = RuskHttpClient::new(cfg.rusk_address.clone());
        let (txs, top) =
            TxInquirer::txs_from_block_range(&client, height_beg, height_end)
                .await?;
        let (total, requests) =
            RequestScanner::scan_transactions_related_to_notes(
                txs,
                note_hashes,
            );
        Ok((requests, top, total))
    }
}
