// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{CitadelInquirer, Error};
pub mod websocket;
use crate::websocket::ws_license_contract_mock_server;

const TEST_DURATION_SECONDS: u64 = 4;
const PORT: u32 = 9126;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn get_licenses() -> Result<(), Error> {
    tokio::spawn(ws_license_contract_mock_server(TEST_DURATION_SECONDS, PORT));
    let url = format!("127.0.0.1:{}", PORT);
    let id = None;
    let block_heights = 0..1024u64;
    let ser_licenses =
        CitadelInquirer::get_licenses(url, id, block_heights).await?;
    assert_eq!(ser_licenses.len(), 2);
    Ok(())
}
