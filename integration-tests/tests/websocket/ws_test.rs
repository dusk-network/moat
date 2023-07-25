// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use futures_util::{SinkExt, StreamExt};
use moat_core::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::client_async;
use crate::websocket::ws_license_contract_mock_server;
use crate::websocket::ws_common::*;

#[tokio::test]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn ws_license_contract_mock_call() -> Result<(), Error> {
    println!("test driver - spawning ws license contract mock server");
    tokio::spawn(ws_license_contract_mock_server(4));
    println!("test driver - spawning websocket client");
    tokio::spawn(send_request_to_ws_server());
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok(())
}

async fn send_request_to_ws_server() {
    let url = "127.0.0.1:9127";

    println!("client - connecting");
    let stream = TcpStream::connect(url)
        .await
        .expect("Connecting to the server should succeed");

    let (mut ws_stream, _) =
        client_async("ws://localhost", stream).await.expect("Failed to connect");
    println!("client - websocket handshake has been successfully completed");

    let id = 93;
    let fn_args = Vec::from(&b"I am call data 0"[..]);
    let request = serde_json::to_string(&ExecutionRequest {
        request_id: Some(id as i32),
        contract: [03; 32], // todo - we need Citadel Contract Id here
        fn_name: "test_function".to_string(),
        fn_args,
    })
        .expect("Serializing request should succeed");

    println!("client - sending request={:?}", id);
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
