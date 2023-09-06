// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::types::{SpentTx2, SpentTxResponse2, TxStatus};
use crate::Error;
use crate::Error::TransactionError;
use dusk_bls12_381::BlsScalar;
use dusk_wallet::{RuskHttpClient, RuskRequest};
use std::time::Duration;
use tokio::time::sleep;

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
        println!("response={:x?}", std::str::from_utf8(response.as_slice()));
        let response =
            serde_json::from_slice::<SpentTxResponse2>(&response)?.tx;
        match response {
            Some(SpentTx2 { err: Some(err), .. }) => {
                println!("status ERR={}", err);
                Ok(TxStatus::Error(err))
            }
            Some(_) => {
                println!("status OK");
                Ok(TxStatus::Ok)
            }
            None => {
                println!("status NOT_FOUND");
                Ok(TxStatus::NotFound)
            }
        }
    }

    pub async fn wait_for(
        client: &RuskHttpClient,
        tx_id: BlsScalar,
    ) -> Result<(), Error> {
        let tx_id = format!("{:x}", tx_id);
        println!("xxx={}", tx_id);
        Self::wait_for_tx(client, tx_id).await
    }

    async fn wait_for_tx(
        client: &RuskHttpClient,
        tx_id: impl AsRef<str>,
    ) -> Result<(), Error> {
        const TIMEOUT_SECS: i32 = 10;
        let mut i = 1;
        while i <= TIMEOUT_SECS {
            let status = Self::tx_status(client, tx_id.as_ref()).await?;

            match status {
                TxStatus::Ok => break,
                TxStatus::Error(err) => {
                    return Err(TransactionError(Box::from(err)))?
                }
                TxStatus::NotFound => {
                    println!("Awaiting ({}) for {}", i, tx_id.as_ref());
                    sleep(Duration::from_millis(1000)).await;
                    i += 1;
                }
            }
        }
        if i > TIMEOUT_SECS {
            Err(TransactionError(Box::from("Confirmation timed out")))
        } else {
            Ok(())
        }
    }
}
