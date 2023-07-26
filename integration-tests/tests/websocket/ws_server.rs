// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::websocket::ws_common::*;
use futures_util::{SinkExt, StreamExt};
use moat_core::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub async fn ws_license_contract_mock_server(
    seconds: u64,
) -> Result<(), Error> {
    let addr = "127.0.0.1:9127".to_string();

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("server - listening on: {}", addr);

    println!("server - accepting requests");
    while let Ok((stream, _)) = listener.accept().await {
        println!("server - spawning accept connection");
        tokio::spawn(accept_connection(stream));
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
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

    println!(
        "server - obtained request={:?} fn_name={}",
        request.request_id, request.fn_name
    );

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
