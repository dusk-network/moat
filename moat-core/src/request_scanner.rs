// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{RequestExtractor, Transactions};
use zk_citadel::license::Request;

pub struct RequestScanner;

impl RequestScanner {
    pub fn scan(txs: Transactions) -> Vec<Request> {
        let mut requests = Vec::new();
        for tx in &txs.transactions {
            match RequestExtractor::extract_request_from_tx(tx) {
                Ok(request) => requests.push(request),
                _ => (),
            }
        }
        requests
    }
}
