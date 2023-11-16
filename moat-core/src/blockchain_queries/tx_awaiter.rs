// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::bc_types::{SpentTx2, SpentTxResponse2, TxStatus};
use crate::Error;
use crate::Error::Transaction;
use dusk_bls12_381::BlsScalar;
use dusk_wallet::{RuskHttpClient, RuskRequest};
use std::time::Duration;
use tokio::time::sleep;
use tracing::trace;

pub struct TxAwaiter;

impl TxAwaiter {
    async fn query(
        client: &RuskHttpClient,
        query: impl AsRef<str>,
    ) -> Result<Vec<u8>, Error> {
        let request =
            RuskRequest::new("gql", query.as_ref().as_bytes().to_vec());
        client
            .call(2, "Chain", &request)
            .await
            .map_err(|e| e.into())
    }

    async fn tx_status(
        client: &RuskHttpClient,
        tx_id: impl AsRef<str>,
    ) -> Result<TxStatus, Error> {
        let query = "query { tx(hash: \"####\") { err }}"
            .replace("####", tx_id.as_ref());
        let response = Self::query(client, &query).await?;
        trace!("response={:x?}", std::str::from_utf8(response.as_slice()));
        let response =
            serde_json::from_slice::<SpentTxResponse2>(&response)?.tx;
        match response {
            Some(SpentTx2 { err: Some(err), .. }) => {
                trace!("status ERR={}", err);
                Ok(TxStatus::Error(err))
            }
            Some(_) => {
                trace!("status OK");
                Ok(TxStatus::Ok)
            }
            None => {
                trace!("status NOT_FOUND");
                Ok(TxStatus::NotFound)
            }
        }
    }

    pub async fn wait_for(
        client: &RuskHttpClient,
        tx_id: BlsScalar,
    ) -> Result<(), Error> {
        let tx_id = hex::encode(tx_id.to_bytes());
        Self::wait_for_tx(client, tx_id).await
    }

    async fn wait_for_tx(
        client: &RuskHttpClient,
        tx_id: impl AsRef<str>,
    ) -> Result<(), Error> {
        const TIMEOUT_SECS: i32 = 30;
        let mut i = 1;
        while i <= TIMEOUT_SECS {
            let status = Self::tx_status(client, tx_id.as_ref()).await?;

            match status {
                TxStatus::Ok => break,
                TxStatus::Error(err) => return Err(Transaction(err.into()))?,
                TxStatus::NotFound => {
                    trace!("Awaiting ({}) for {}", i, tx_id.as_ref());
                    sleep(Duration::from_millis(1000)).await;
                    i += 1;
                }
            }
        }
        if i > TIMEOUT_SECS {
            Err(Transaction("Confirmation timed out".into()))
        } else {
            Ok(())
        }
    }
}
