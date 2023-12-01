// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use zk_citadel_moat::{Error, JsonLoader, PayloadExtractor, Tx};
use zk_citadel::license::Request;

#[test]
fn extract_request_not_present() -> Result<(), Error> {
    let tx_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/tx_no_request.json");
    let tx = Tx::from_file(tx_path)?;

    let result = PayloadExtractor::payload_from_tx::<Request>(&tx);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn extract_call_data_not_present() -> Result<(), Error> {
    let tx_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/tx_no_call_data.json");
    let tx = Tx::from_file(tx_path)?;

    let result = PayloadExtractor::payload_from_tx::<Request>(&tx);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn extract_request_present() -> Result<(), Error> {
    let tx_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/tx_with_request.json");
    let tx = Tx::from_file(tx_path)?;

    let result = PayloadExtractor::payload_from_tx::<Request>(&tx);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn extract_bad_payload() -> Result<(), Error> {
    let tx_path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tx/tx_bad_payload.json");
    let tx = Tx::from_file(tx_path)?;

    let result = PayloadExtractor::payload_from_tx::<Request>(&tx);
    assert!(result.is_err());
    Ok(())
}
