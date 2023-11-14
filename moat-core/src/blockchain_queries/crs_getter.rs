// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Error;
use dusk_wallet::{RuskHttpClient, RuskRequest};
use reqwest::Response;
use sha2::{Digest, Sha256};

pub struct CrsGetter;

type CRSHash = [u8; 32];
const CRS_HASH_HEADER: &str = "crs-hash";

impl CrsGetter {
    pub async fn get_crs(client: &RuskHttpClient) -> Result<Vec<u8>, Error> {
        let crs_request = RuskRequest::new("crs", vec![]);
        let result = client.call_raw(2, "rusk", &crs_request, false).await;
        match result {
            Ok(response) => {
                let received_hash = Self::hash_from_header(&response)?;
                let crs = response.bytes().await?;
                let this_hash = Self::hash_of_bytes(crs.as_ref());
                if received_hash != this_hash {
                    return Err(Error::CRS(Box::from("corrupted CRS")));
                }
                Ok(crs.to_vec())
            }
            Err(err) => Err(err.into()),
        }
    }

    fn hash_from_header(response: &Response) -> Result<CRSHash, Error> {
        let crs_hash = response
            .headers()
            .get(CRS_HASH_HEADER)
            .ok_or(Error::CRS(Box::from("missing CRS hash header")))?;
        let crs_hash = crs_hash.to_str().map_err(|_| {
            Error::CRS(Box::from("failed CRS hash header string conversion"))
        })?;
        let mut h = CRSHash::default();
        hex::decode_to_slice(crs_hash, h.as_mut_slice())?;
        Ok(h)
    }

    fn hash_of_bytes<T: AsRef<[u8]>>(bytes: T) -> CRSHash {
        let mut hasher = Sha256::new();
        hasher.update(bytes.as_ref());
        hasher.finalize().into()
    }
}
