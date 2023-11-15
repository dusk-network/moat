// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::contract_queries::block::BlockInPlace;
use crate::error::Error;
use crate::Error::InvalidQueryResponse;
use crate::MAX_CALL_SIZE;
use bytecheck::CheckBytes;
use bytes::Bytes;
use dusk_wallet::{RuskHttpClient, RuskRequest};
use phoenix_core::transaction::ModuleId;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{check_archived_root, Archive, Deserialize, Infallible};

pub struct ContractInquirer {}

impl ContractInquirer {
    pub async fn query_contract<A, R>(
        client: &RuskHttpClient,
        args: A,
        contract_id: ModuleId,
        method: impl AsRef<str>,
    ) -> Result<R, Error>
    where
        A: Archive,
        A: rkyv::Serialize<
            rkyv::ser::serializers::AllocSerializer<MAX_CALL_SIZE>,
        >,
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let contract_id = hex::encode(contract_id.as_slice());
        let response = client
            .contract_query::<A, MAX_CALL_SIZE>(
                contract_id.as_ref(),
                method.as_ref(),
                &args,
            )
            .await?;

        let response_data = check_archived_root::<R>(response.as_slice())
            .map_err(|_| {
                InvalidQueryResponse(Box::from("rkyv deserialization error"))
            })?;
        let r = response_data
            .deserialize(&mut Infallible)
            .expect("Infallible");
        Ok(r)
    }

    pub async fn query_contract_with_feeder<A>(
        client: &RuskHttpClient,
        args: A,
        contract_id: ModuleId,
        method: impl AsRef<str>,
    ) -> Result<
        impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>,
        Error,
    >
    where
        A: Archive,
        A: rkyv::Serialize<
            rkyv::ser::serializers::AllocSerializer<MAX_CALL_SIZE>,
        >,
    {
        let contract_id = hex::encode(contract_id.as_slice());
        let req = rkyv::to_bytes(&args).map_err(|_| Error::SerRkyv)?.to_vec();
        let stream = client
            .call_raw(
                1,
                contract_id.as_ref(),
                &RuskRequest::new(method.as_ref(), req),
                true,
            )
            .wait()?
            .bytes_stream();
        Ok(stream)
    }
}
