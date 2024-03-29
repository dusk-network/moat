// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use zk_citadel_moat::ContractInquirerWs;

pub async fn send_request_to_ws_server(port: u32) {
    let url = format!("127.0.0.1:{}", port);
    let _result: () = ContractInquirerWs::query_contract(
        url,
        None,
        (),
        [03; 32],
        "some_method",
    )
    .await
    .expect("Contract query should succeed");
}
