// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{Error, RequestExtractor, Tx};

#[test]
fn extract_request_not_present() -> Result<(), Error> {
    let tx_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/tx_no_request.json");
    let tx = Tx::from_file(tx_path)?;

    let result = RequestExtractor::extract_request_from_tx(&tx);
    println!("result={:?}", result);
    assert!(result.is_err());
    Ok(())
}
