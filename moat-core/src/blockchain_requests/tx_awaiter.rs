// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Error, SpentTx, SpentTxResponse, TxStatus};
use dusk_wallet::{RuskHttpClient, RuskRequest};
use std::time::Duration;
use tokio::time::sleep;

pub struct TxAwaiter;

impl TxAwaiter {
    async fn query(
        client: &RuskHttpClient,
        query: &str,
    ) -> Result<Vec<u8>, Error> {
        let request = RuskRequest::new("gql", query.as_bytes().to_vec());
        client.call(2, "Chain", &request).await.map_err(|e|e.into())
    }

    async fn tx_status(client: &RuskHttpClient, tx_id: &str) -> Result<TxStatus, Error> {
        let query =
            "query { tx(hash: \"####\") { err }}".replace("####", tx_id);
        let response = Self::query(client, &query).await?;
        let response = serde_json::from_slice::<SpentTxResponse>(&response)?.tx;

        match response {
            Some(SpentTx {
                txerror: Some(err), ..
            }) => Ok(TxStatus::Error(err)),
            Some(_) => Ok(TxStatus::Ok),

            None => Ok(TxStatus::NotFound),
        }
    }

    pub async fn wait_for(client: &RuskHttpClient, tx_id: &str) -> Result<(), Error> {
        const TIMEOUT_SECS: i32 = 30;
        let mut i = 1;
        while i <= TIMEOUT_SECS {
            let status = Self::tx_status(client, tx_id).await?;

            match status {
                TxStatus::Ok => break,
                TxStatus::Error(err) => return Err(Error::Transaction(err))?,
                TxStatus::NotFound => {
                    (self.status)(
                        format!(
                            "Waiting for confirmation... ({}/{})",
                            i, TIMEOUT_SECS
                        )
                        .as_str(),
                    );
                    sleep(Duration::from_millis(1000)).await;
                    i += 1;
                }
            }
        }
        Ok(())
    }
}
