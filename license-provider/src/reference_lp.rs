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
    vk_lp: ViewKey,
}

impl ReferenceLP {
    pub fn new(ssk: SecretSpendKey) -> Self {
        Self { vk_lp: ssk.view_key() }
    }

    pub fn init<P: AsRef<Path>>(lp_config_path: P) -> Result<Self, Error> {
        let lp_config: LPConfig = LPConfig::from_file(lp_config_path)?;
        let ssk_bytes = hex::decode(lp_config.ssk_lp)?;
        let ask = SecretSpendKey::from_slice(ssk_bytes.as_slice())?;
        Ok(Self::new(ask))
    }

    /// scans the entire blockchain for relevant requests
    pub async fn run(&self, cfg: &BlockchainAccessConfig) -> Result<(), Error> {
        let mut height = 0;
        loop {
            let height_end = height + 10000;
            let (requests, top) =
                RequestScanner::scan_block_range(height, height_end, &cfg)
                    .await?;

            let relevant_requests = self.relevant_requests(&requests)?;

            println!(
                "found {} requests in block range ({},{}), relevant: {}",
                requests.len(),
                height,
                height_end,
                relevant_requests.len()
            );

            // todo: hook up further processing of relevant requests here

            if top <= height_end {
                return Ok(());
            }

            height = height_end;
        }
    }

    /// Given a collection of requests, returns a new collection
    /// containing only requests relevant to `this` license provider
    pub fn relevant_requests(
        &self,
        requests: &Vec<Request>,
    ) -> Result<Vec<Request>, Error> {
        let mut relevant_requests: Vec<Request> = Vec::new();
        for request in requests.iter() {
            if self.is_relevant_request(&request) {
                let r = Request {
                    ..*request
                };
                relevant_requests.push(r);
            }
        }
        Ok(relevant_requests)
    }

    pub fn is_relevant_request(&self, request: &Request) -> bool {
        self.vk_lp.owns(&request.rsa)
    }
}
