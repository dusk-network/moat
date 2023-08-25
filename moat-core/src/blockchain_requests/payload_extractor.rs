// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{check_archived_root, Archive, Deserialize, Infallible};

use crate::error::Error;
use crate::types::{Tx, Tx2, TxJson, TxJson2};
use crate::Error::PayloadNotPresent;
use base64::{engine::general_purpose, Engine as _};
use bytecheck::CheckBytes;
use phoenix_core::Transaction;
use rkyv::validation::validators::DefaultValidator;

pub struct PayloadExtractor;

impl PayloadExtractor {
    pub fn payload_from_tx<P>(tx: &Tx2) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let r = tx.call_data.as_ref().unwrap().data.as_str(); // todo: take care of unwrap
        Self::payload_from_tx_json::<P, _>(r)
    }

    pub fn payload_from_tx_json<P, S>(payload_ser: S) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
        S: AsRef<str>,
    {
        let mut payload_ser = hex::decode(payload_ser.as_ref()).unwrap(); // todo: unwrap
        println!("ser={}", hex::encode(payload_ser.clone()));

        let payload = check_archived_root::<P>(&payload_ser).map_err(|_| {
            PayloadNotPresent(Box::from("rkyv deserialization error"))
        })?;
        let p: P = payload.deserialize(&mut Infallible).expect("Infallible");
        Ok(p)
    }
}
