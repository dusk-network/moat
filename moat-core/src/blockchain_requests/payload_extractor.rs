// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{check_archived_root, Archive, Deserialize, Infallible};

use crate::error::Error;
use crate::types::{Tx, TxJson};
use crate::Error::PayloadNotPresent;
use base64::{engine::general_purpose, Engine as _};
use bytecheck::CheckBytes;
use rkyv::validation::validators::DefaultValidator;

pub struct PayloadExtractor;

impl PayloadExtractor {
    pub fn payload_from_tx<P>(tx: &Tx) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let tx_json: TxJson = serde_json::from_str(tx.json.as_str())?;
        Self::payload_from_tx_json::<P>(&tx_json)
    }

    pub fn payload_from_tx_json<P>(tx_json: &TxJson) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let payload_base64 = &tx_json.call.CallData;
        let payload_ser = general_purpose::STANDARD.decode(payload_base64)?;

        let payload = check_archived_root::<P>(payload_ser.as_slice())
            .map_err(|_| {
                PayloadNotPresent(Box::from("rkyv deserialization error"))
            })?;
        let p: P = payload.deserialize(&mut Infallible).expect("Infallible");
        Ok(p)
    }
}
