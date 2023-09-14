// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::types::*;
use dusk_wallet::{RuskHttpClient, RuskRequest};

pub struct BcInquirer;

impl BcInquirer {
    pub async fn gql_query(
        client: &RuskHttpClient,
        query: impl AsRef<str>,
    ) -> Result<Vec<u8>, dusk_wallet::Error> {
        let request =
            RuskRequest::new("gql", query.as_ref().as_bytes().to_vec());
        client.call(2, "Chain", &request).await
    }

    pub async fn block_height(client: &RuskHttpClient) -> Result<u64, Error> {
        let query = "query { block(height: -1) {header { height}} }";
        let response = Self::gql_query(client, query).await?;
        let result = serde_json::from_slice::<QueryResult2>(&response)?;
        Ok(result.block.header.height)
    }
}
