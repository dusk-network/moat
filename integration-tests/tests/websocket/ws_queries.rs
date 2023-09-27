// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use moat_core::{CitadelInquirerWs, Error, LicenseSession, LicenseSessionId};
use dusk_jubjub::BlsScalar;

const TEST_DURATION_SECONDS: u64 = 4;
const PORT: u32 = 9126;

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn contract_ws_queries() -> Result<(), Error> {
    const NUM_TESTS: u32 = 2;
    tokio::spawn(ws_license_contract_mock_multi_server(
        TEST_DURATION_SECONDS,
        PORT,
        NUM_TESTS,
    ));
    ws_query_licenses().await?;
    ws_query_session().await?;
    Ok(())
}

async fn ws_query_licenses() -> Result<(), Error> {
    let url = format!("127.0.0.1:{}", PORT);
    let id = None;
    let block_heights = 0..1024u64;
    let ser_licenses =
        CitadelInquirerWs::get_licenses(url, id, block_heights).await?;
    assert_eq!(ser_licenses.len(), 2);
    Ok(())
}

async fn ws_query_session() -> Result<(), Error> {
    let url = format!("127.0.0.1:{}", PORT);
    let id = None;
    let session_id = LicenseSessionId {
        id: BlsScalar::zero(),
    };
    let session: Option<LicenseSession> =
        CitadelInquirerWs::get_session(url, id, session_id).await?;
    assert!(session.is_some());
    let public_inputs = &session.as_ref().unwrap().public_inputs;
    assert_eq!(public_inputs.len(), 1);
    assert_eq!(public_inputs[0], BlsScalar::zero());
    Ok(())
}
