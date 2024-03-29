// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::utils::MoatCoreUtils;
use crate::wallet_accessor::BlockchainAccessConfig;
use crate::{Error, JsonLoader, RequestScanner, MAX_REQUEST_SIZE};
use blake3::OUT_LEN;
use dusk_bytes::DeserializableSlice;
use dusk_pki::{PublicSpendKey, SecretSpendKey, ViewKey};
use std::collections::BTreeSet;
use std::path::Path;
use zk_citadel::license::Request;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct LPConfig {
    pub psk_lp: String,
    pub ssk_lp: String,
}
impl JsonLoader for LPConfig {}

const BLOCKS_RANGE_LEN: u64 = 10000;

pub struct ReferenceLP {
    pub psk_lp: PublicSpendKey,
    pub ssk_lp: SecretSpendKey,
    pub vk_lp: ViewKey,
    pub requests_to_process: Vec<Request>,
    pub requests_hashes: BTreeSet<[u8; OUT_LEN]>,
}

impl ReferenceLP {
    fn new(
        psk_lp: PublicSpendKey,
        ssk_lp: SecretSpendKey,
        vk_lp: ViewKey,
    ) -> Self {
        Self {
            psk_lp,
            ssk_lp,
            vk_lp,
            requests_to_process: Vec::new(),
            requests_hashes: BTreeSet::new(),
        }
    }

    pub fn create<P: AsRef<Path>>(lp_config_path: P) -> Result<Self, Error> {
        let lp_config: LPConfig = LPConfig::from_file(lp_config_path)?;

        let ssk_bytes = hex::decode(lp_config.ssk_lp)?;
        let ssk_lp = SecretSpendKey::from_slice(ssk_bytes.as_slice())?;

        Self::create_with_ssk(&ssk_lp)
    }

    pub fn create_with_ssk(ssk_lp: &SecretSpendKey) -> Result<Self, Error> {
        let psk_lp = ssk_lp.public_spend_key();
        let vk_lp = ssk_lp.view_key();
        Ok(Self::new(psk_lp, *ssk_lp, vk_lp))
    }

    /// Scans the entire blockchain for the requests to process.
    /// Returns total number of requests found and number of requests addressed
    /// to this LP.
    pub async fn scan(
        &mut self,
        cfg: &BlockchainAccessConfig,
    ) -> Result<(usize, usize), Error> {
        let mut height = 0;
        let mut total = 0usize;
        let mut total_owned = 0usize;
        loop {
            let height_end = height + BLOCKS_RANGE_LEN;
            let (requests, top) =
                RequestScanner::scan_block_range(height, height_end, cfg)
                    .await?;
            total += requests.len();
            let owned_requests = self.retain_owned_requests(requests);
            for owned_request in owned_requests {
                if self.insert_request(owned_request) {
                    total_owned += 1;
                }
            }
            if top <= height_end {
                return Ok((total, total_owned));
            }
            height = height_end;
        }
    }

    /// Scans last n blocks for the requests to process.
    /// Returns the total number of requests found and the number of requests
    /// addressed to this LP.
    pub async fn scan_last_blocks(
        &mut self,
        n: usize,
        cfg: &BlockchainAccessConfig,
    ) -> Result<(usize, usize), Error> {
        let mut total = 0usize;
        let mut total_owned = 0usize;
        let requests = RequestScanner::scan_last_blocks(n, cfg).await?;
        total += requests.len();
        let owned_requests = self.retain_owned_requests(requests);
        for owned_request in owned_requests {
            if self.insert_request(owned_request) {
                total_owned += 1;
            }
        }
        Ok((total, total_owned))
    }

    /// Given a collection of requests, retain only those requests
    /// in the collection which are owned by 'this' license provider.
    pub fn retain_owned_requests(
        &self,
        mut requests: Vec<Request>,
    ) -> Vec<Request> {
        requests.retain(|request| self.is_owned_request(request));
        requests
    }

    fn is_owned_request(&self, request: &Request) -> bool {
        self.vk_lp.owns(&request.rsa)
    }

    fn insert_request(&mut self, request: Request) -> bool {
        let hash = Self::hash_request(&request);
        if self.requests_hashes.insert(hash) {
            self.requests_to_process.push(request);
            true
        } else {
            false
        }
    }

    /// Take and remove one of the requests to process.
    pub fn take_request(&mut self) -> Option<Request> {
        self.requests_to_process.pop().map(|request| {
            self.requests_hashes.remove(&Self::hash_request(&request));
            request
        })
    }

    /// Retrieve request with a given request hash, or None if not found.
    pub fn get_request(&mut self, request_hash: &String) -> Option<Request> {
        for (index, request) in self.requests_to_process.iter().enumerate() {
            if MoatCoreUtils::to_hash_hex(request) == *request_hash {
                self.requests_hashes.remove(&Self::hash_request(request));
                return Some(self.requests_to_process.remove(index));
            }
        }
        None
    }

    fn hash_request(request: &Request) -> [u8; OUT_LEN] {
        *blake3::hash(
            rkyv::to_bytes::<_, MAX_REQUEST_SIZE>(request)
                .expect("Serializing should be infallible")
                .as_slice(),
        )
        .as_bytes()
    }
}
