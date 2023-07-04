// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::DeserializableSlice;
use dusk_pki::{SecretSpendKey, ViewKey};
use moat_core::{Error, JsonLoader, RequestScanner};
use std::path::Path;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct LPConfig {
    pub ssk_lp: String,
}
impl JsonLoader for LPConfig {}

pub struct ReferenceLP {
    pub vk_lp: ViewKey,
    pub requests_to_process: Vec<Request>,
}

impl ReferenceLP {
    fn new(ssk: SecretSpendKey) -> Self {
        Self {
            vk_lp: ssk.view_key(),
            requests_to_process: Vec::new(),
        }
    }

    pub fn init<P: AsRef<Path>>(lp_config_path: P) -> Result<Self, Error> {
        let lp_config: LPConfig = LPConfig::from_file(lp_config_path)?;
        let ssk_bytes = hex::decode(lp_config.ssk_lp)?;
        let ask = SecretSpendKey::from_slice(ssk_bytes.as_slice())?;
        Ok(Self::new(ask))
    }

    /// scans the entire blockchain for requests to process
    pub async fn scan(&mut self, cfg: &BlockchainAccessConfig) -> Result<(), Error> {
        let mut height = 0;
        loop {
            let height_end = height + 10000;
            let (requests, top) =
                RequestScanner::scan_block_range(height, height_end, &cfg)
                    .await?;

            let owned_requests = self.filter_owned_requests(&requests)?;

            println!(
                "found {} requests in block range ({},{}), owned: {}",
                requests.len(),
                height,
                height_end,
                owned_requests.len()
            );

            self.requests_to_process.extend(owned_requests);
            
            if top <= height_end {
                return Ok(());
            }

            height = height_end;
        }
    }

    /// Given a collection of requests, returns a new collection
    /// containing only requests relevant to `this` license provider
    pub fn filter_owned_requests(
        &self,
        requests: &Vec<Request>,
    ) -> Result<Vec<Request>, Error> {
        let mut relevant_requests: Vec<Request> = Vec::new();
        for request in requests.iter() {
            if self.is_owned_request(&request) {
                let r = Request {
                    ..*request
                };
                relevant_requests.push(r);
            }
        }
        Ok(relevant_requests)
    }

    fn is_owned_request(&self, request: &Request) -> bool {
        self.vk_lp.owns(&request.rsa)
    }
}
