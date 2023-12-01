// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use zk_citadel_moat::{JsonLoader, RequestScanner, Transactions};

#[test]
fn scan_transactions() {
    let txs_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/transactions.json");

    let txs = Transactions::from_file(txs_path)
        .expect("transactions file should load correctly");

    let requests = RequestScanner::scan_transactions(txs);

    const NUM_EXPECTED_REQUESTS: usize = 9;

    assert_eq!(requests.len(), NUM_EXPECTED_REQUESTS);
}
