// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::contract_queries::ws_types::{ExecutionRequest, ExecutionResponse};
use crate::error::Error;
use crate::Error::{InvalidQueryResponse, WebSocketStreamClosed};
use bytecheck::CheckBytes;
use futures_util::{SinkExt, StreamExt};
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{check_archived_root, Archive, Deserialize, Infallible};
use tokio::net::TcpStream;
use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::Message;

#[allow(dead_code)]
pub struct ContractInquirerWs {}

#[allow(dead_code)]
const MAX_CALL_SIZE: usize = 65536;

#[allow(dead_code)]
impl ContractInquirerWs {
    pub async fn query_contract<A, R>(
        url: impl AsRef<str>,
        id: Option<i32>,
        args: A,
        contract_id: ModuleId,
        method: impl AsRef<str>,
    ) -> Result<R, Error>
    where
        A: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let stream = TcpStream::connect(url.as_ref()).await?;

        let url = format!("ws://{}", url.as_ref()); // todo: find a more elegant way // /01/stream

        let (mut ws_stream, _) = client_async(url, stream).await?;

        let fn_args = rkyv::to_bytes::<_, MAX_CALL_SIZE>(&args)
            .expect("Request should serialize correctly")
            .to_vec();
        let request = serde_json::to_string(&ExecutionRequest {
            request_id: id,
            contract: contract_id,
            fn_name: method.as_ref().to_string(),
            fn_args,
        })?;

        ws_stream.send(Message::Text(request)).await?;

        let msg = ws_stream.next().await.ok_or(WebSocketStreamClosed)??;

        let msg = match msg {
            Message::Text(msg) => msg,
            _ => panic!("Shouldn't receive anything but text"),
        };

        let response: ExecutionResponse = serde_json::from_str(&msg)?;
        if let Some(response_error) = response.error {
            return Err(InvalidQueryResponse(Box::from(response_error)));
        }
        if let Some(sent_id) = id {
            match response.request_id {
                Some(received_id) if sent_id == received_id => (),
                _ => {
                    return Err(InvalidQueryResponse(Box::from(
                        "received wrong request id",
                    )))
                }
            }
        }
        let response_data = check_archived_root::<R>(response.data.as_slice())
            .map_err(|_| {
                InvalidQueryResponse(Box::from("rkyv deserialization error"))
            })?;
        let r: R = response_data
            .deserialize(&mut Infallible)
            .expect("Infallible");
        Ok(r)
    }
}
