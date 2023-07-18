// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::websocket::ws_common::*;
use futures_util::{SinkExt, StreamExt};
use moat_core::ContractInquirer;
use tokio::net::TcpStream;
use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::Message;

#[allow(dead_code)]
pub async fn send_request_to_ws_server_2() {
    let url = "127.0.0.1:9127";

    println!("client - connecting");
    let stream = TcpStream::connect(url)
        .await
        .expect("Connecting to the server should succeed");

    let (mut ws_stream, _) = client_async("ws://localhost", stream)
        .await
        .expect("Failed to connect");
    println!("client - websocket handshake has been successfully completed");

    let id = 93;
    let fn_name = "get_licenses".to_string();
    let fn_args = Vec::from(&b"argument for get_licenses"[..]);
    let request = serde_json::to_string(&ExecutionRequest {
        request_id: Some(id as i32),
        contract: [03; 32], // todo - we need Citadel Contract Id here
        fn_name: fn_name.clone(),
        fn_args,
    })
    .expect("Serializing request should succeed");

    println!("client - sending request id={:?} fn_name={}", id, fn_name);
    ws_stream
        .send(Message::Text(request))
        .await
        .expect("Sending request to the server should succeed");

    let msg = ws_stream
        .next()
        .await
        .expect("Stream shouldn't close while awaiting responses")
        .expect("Response should be received without error");

    let msg = match msg {
        Message::Text(msg) => msg,
        _ => panic!("Shouldn't receive anything but text"),
    };

    let response: ExecutionResponse = serde_json::from_str(&msg)
        .expect("Response should deserialize successfully");
    println!("client - obtained response={:?}", response.request_id);
    assert_eq!(response.request_id, Some(93));
}

pub async fn send_request_to_ws_server() {
    let _result: () = ContractInquirer::query_contract(
        "127.0.0.1:9127",
        None,
        (),
        [03; 32],
        "some_method",
    )
    .await
    .expect("Contract query should succeed");
}
