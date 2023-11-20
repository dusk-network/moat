// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::license_provider::ReferenceLP;
use moat_core::{Error, JsonLoader, RequestScanner, Transactions};

#[test]
fn lp_filter_requests() -> Result<(), Error> {
    let lp_config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/config/test_keys_lp.json"
    );
    let reference_lp = ReferenceLP::create(&lp_config_path)?;

    let txs_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/transactions.json");

    let txs = Transactions::from_file(txs_path)
        .expect("transactions file should load correctly");

    let requests = RequestScanner::scan_transactions(txs);
    assert_eq!(requests.len(), 9);

    let owned_requests = reference_lp.retain_owned_requests(requests);
    assert_eq!(owned_requests.len(), 2);

    Ok(())
}
