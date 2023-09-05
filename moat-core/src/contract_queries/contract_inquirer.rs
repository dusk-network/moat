// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::Error::InvalidQueryResponse;
use bytecheck::CheckBytes;
use dusk_wallet::RuskHttpClient;
use phoenix_core::transaction::ModuleId;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{check_archived_root, Archive, Deserialize, Infallible};

#[allow(dead_code)]
pub struct ContractInquirer {}

#[allow(dead_code)]
const MAX_CALL_SIZE: usize = 65536;

#[allow(dead_code)]
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
        println!("contract_id={}", contract_id);
        let response = client
            .contract_query::<A, MAX_CALL_SIZE>(
                contract_id.as_ref(),
                method.as_ref(),
                &args,
            )
            .await?;

        println!("response len={}", response.len());

        let response_data = check_archived_root::<R>(response.as_slice())
            .map_err(|_| {
                InvalidQueryResponse(Box::from("rkyv deserialization error"))
            })?;
        let r = response_data
            .deserialize(&mut Infallible)
            .expect("Infallible");
        Ok(r)
    }
}
