// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::websocket::ws_common::*;
use dusk_bls12_381::BlsScalar;
use futures_util::{SinkExt, StreamExt};
use moat_core::{Error, LicenseSession, MAX_RESPONSE_SIZE};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::trace;

pub async fn ws_license_contract_mock_server(
    seconds: u64,
    port: u32,
) -> Result<(), Error> {
    let addr = format!("127.0.0.1:{}", port);

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    trace!("server - listening on: {}", addr);

    trace!("server - accepting requests");
    while let Ok((stream, _)) = listener.accept().await {
        trace!("server - spawning accept connection");
        tokio::spawn(accept_connection(stream));
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        break;
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn ws_license_contract_mock_multi_server(
    seconds: u64,
    port: u32,
    limit: u32,
) -> Result<(), Error> {
    let addr = format!("127.0.0.1:{}", port);

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    trace!("server - listening on: {}", addr);

    trace!("server - accepting requests");
    let mut count = 0u32;
    while let Ok((stream, _)) = listener.accept().await {
        trace!("server - spawning accept connection");
        tokio::spawn(accept_connection(stream));
        count += 1;
        if count == limit {
            break;
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    trace!("server - peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    trace!("server - new websocket connection: {}", addr);

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

    trace!(
        "server - obtained request={:?} fn_name={}",
        request.request_id,
        request.fn_name
    );

    let response_id = request.request_id;

    let data: Vec<u8> = match request.fn_name.as_str() {
        "get_session" => {
            let response_data: Option<LicenseSession> = Some(LicenseSession {
                public_inputs: vec![BlsScalar::zero()],
            });
            rkyv::to_bytes::<_, MAX_RESPONSE_SIZE>(&response_data)
                .expect("Data should serialize correctly")
                .to_vec()
        }
        "get_licenses" => {
            let response_data = vec![vec![1u8], vec![2u8]];
            rkyv::to_bytes::<_, MAX_RESPONSE_SIZE>(&response_data)
                .expect("Data should serialize correctly")
                .to_vec()
        }
        _ => vec![],
    };

    let response = serde_json::to_string(&ExecutionResponse {
        request_id: response_id,
        data,
        error: None,
    })
    .expect("Serializing response should succeed");

    trace!("server - sending response ={:?}", response_id);
    ws_stream
        .send(Message::Text(response))
        .await
        .expect("Sending request to the server should succeed");
}
