// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use futures_util::{SinkExt, StreamExt};
use moat_core::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::client_async;

/// ================================================================================================

/// A request sent by the websocket client, asking for a specific contract
/// function to be executed with the given arguments.
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize)]
struct ExecutionRequest {
    /// The request ID, allowing for differentiating multiple in-flight
    /// requests.
    request_id: Option<i32>,
    /// The contract to call.
    #[serde_as(as = "serde_with::hex::Hex")]
    contract: [u8; 32],
    /// The function name to call in the contract.
    fn_name: String,
    /// The arguments to pass to the function.
    #[serde_as(as = "serde_with::hex::Hex")]
    fn_args: Vec<u8>,
}

/// Response to a [`ExecutionRequest`] with the same `request_id`.
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize)]
struct ExecutionResponse {
    /// The request ID, allowing for differentiating multiple in-flight
    /// requests.
    request_id: Option<i32>,
    /// The data returned by the contract call.
    #[serde_as(as = "serde_with::hex::Hex")]
    data: Vec<u8>,
    /// A possible error happening during the contract call.
    error: Option<String>,
}

#[derive(Debug)]
enum ExecutionError {
    Json(serde_json::Error),
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Json(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<serde_json::Error> for ExecutionError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// ================================================================================================

#[tokio::test]
#[cfg_attr(not(feature = "int_tests"), ignore)]
async fn websocket_call() -> Result<(), Error> {
    let addr = "127.0.0.1:9127".to_string();

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("server - listening on: {}", addr);
    println!("test driver - spawning websocket client");
    tokio::spawn(send_request_to_ws_server());

    println!("server - accepting requests");
    while let Ok((stream, _)) = listener.accept().await {
        println!("server - spawning accept connection");
        tokio::spawn(accept_connection(stream));
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        break;
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("server - peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("server - new websocket connection: {}", addr);

    let msg = ws_stream
        .next()
        .await
        .expect("Stream shouldn't close while awaiting responses")
        .expect("Response should be received without error");

    let msg = match msg {
        Message::Text(msg) => msg,
        _ => panic!("Shouldn't receive anything but text"),
    };

    let request: ExecutionRequest = serde_json::from_str(&msg)
        .expect("Request should deserialize successfully");

    println!("server - obtained request={:?}", request.request_id);

    let response_id = request.request_id;
    let response = serde_json::to_string(&ExecutionResponse {
        request_id: response_id,
        data: Vec::new(), // todo
        error: None,
    })
    .expect("Serializing response should succeed");

    println!("server - sending response ={:?}", response_id);
    ws_stream
        .send(Message::Text(response))
        .await
        .expect("Sending request to the server should succeed");
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
