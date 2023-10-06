// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{check_archived_root, Archive, Deserialize, Infallible};

use crate::bc_types::Tx;
use crate::error::Error;
use crate::Error::PayloadNotPresent;
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
        let r = tx
            .call_data
            .as_ref()
            .ok_or(PayloadNotPresent(Box::from("missing call data")))?
            .data
            .as_str();
        Self::payload_from_call_data::<P, _>(r)
    }

    fn payload_from_call_data<P, S>(payload_ser: S) -> Result<P, Error>
    where
        P: Archive,
        P::Archived: Deserialize<P, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
        S: AsRef<str>,
    {
        let payload_ser = hex::decode(payload_ser.as_ref())?;

        let payload = check_archived_root::<P>(&payload_ser).map_err(|_| {
            PayloadNotPresent(Box::from("deserialization error"))
        })?;
        let p: P = payload.deserialize(&mut Infallible).expect("Infallible");
        Ok(p)
    }
}