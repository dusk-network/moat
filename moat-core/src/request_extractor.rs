// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{check_archived_root, Deserialize, Infallible};

use crate::error::Error;
use crate::retrieval_types::{Tx, TxJson};
use crate::Error::RequestNotPresent;
use base64::{engine::general_purpose, Engine as _};
use zk_citadel::license::Request;

pub struct RequestExtractor;

impl RequestExtractor {
    pub fn extract_request_from_tx(tx: &Tx) -> Result<Request, Error> {
        let tx_json: TxJson = serde_json::from_str(tx.json.as_str())?;
        let payload_base64 = tx_json.call.CallData;
        let payload_ser = general_purpose::STANDARD
            .decode(payload_base64)
            .map_err(|_| {
                RequestNotPresent(Box::from("base64 decoding error"))
            })?;
        let payload = check_archived_root::<Request>(payload_ser.as_slice())
            .map_err(|_| {
                RequestNotPresent(Box::from("rkyv deserialization error"))
            })?;
        let request: Request =
            payload.deserialize(&mut Infallible).expect("Infallible");
        Ok(request)
    }
}
