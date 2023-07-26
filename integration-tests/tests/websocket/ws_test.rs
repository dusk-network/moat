// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::websocket::{
    send_request_to_ws_server, ws_license_contract_mock_server,
};
use moat_core::Error;

const TEST_DURATION_SECONDS: u64 = 4;
const PORT: u32 = 9125;

#[tokio::test]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn ws_license_contract_mock_call() -> Result<(), Error> {
    println!("test driver - spawning ws license contract mock server");
    tokio::spawn(ws_license_contract_mock_server(TEST_DURATION_SECONDS, PORT));
    println!("test driver - spawning websocket client");
    tokio::spawn(send_request_to_ws_server(PORT));
    tokio::time::sleep(std::time::Duration::from_secs(
        TEST_DURATION_SECONDS + 1,
    ))
    .await;
    Ok(())
}
