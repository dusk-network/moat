// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_pki::PublicSpendKey;
use moat_core::{Error, JsonLoader, RequestScanner};
use std::path::Path;
use wallet_accessor::BlockchainAccessConfig;
use zk_citadel::license::Request;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct LPConfig {
    pub psk: String,
}
impl JsonLoader for LPConfig {}

pub struct ReferenceLP {
    psk: PublicSpendKey,
}

impl ReferenceLP {
    pub fn new(psk: PublicSpendKey) -> Self {
        Self { psk }
    }

    pub fn init<P: AsRef<Path>>(lp_config_path: P) -> Result<Self, Error> {
        let lp_config: LPConfig = LPConfig::from_file(lp_config_path)?;
        let psk_bytes = hex::decode(lp_config.psk)?;
        let psk = PublicSpendKey::from_slice(psk_bytes.as_slice())?;
        Ok(Self::new(psk))
    }

    pub async fn run(&self, cfg: &BlockchainAccessConfig) -> Result<(), Error> {
        let mut height = 0;
        loop {
            let height_end = height + 10000;
            let (requests, top) =
                RequestScanner::scan_block_range(height, height_end, &cfg)
                    .await?;

            println!(
                "{} requests in range ({},{}) top={}",
                requests.len(),
                height,
                height_end,
                top
            );

            self.process_requests(&requests)?;

            if top <= height_end {
                return Ok(());
            }

            height = height_end;
        }
    }

    pub fn process_requests(
        &self,
        _requests: &Vec<Request>,
    ) -> Result<(), Error> {
        // for request in requests {
        //     println!("to me={}", request_addressed_to_this_lp(&request));
        // }
        println!("process_requests, psk={}", hex::encode(self.psk.to_bytes()));
        Ok(())
    }

    // pub fn request_addressed_to_this_lp(&self, ) -> bool {
    //
    // }
}
