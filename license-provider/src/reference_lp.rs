// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use wallet_accessor::BlockchainAccessConfig;
use moat_core::{Error, RequestScanner};

pub struct ReferenceLP;

impl ReferenceLP {
    pub async fn run(cfg: &BlockchainAccessConfig) -> Result<(), Error> {
        let mut height = 0;
        loop {
            let height_end = height + 10000;
            let (requests, top) =
                RequestScanner::scan_block_range(height, height_end, &cfg).await?;

            println!(
                "{} requests in range ({},{}) top={}",
                requests.len(),
                height,
                height_end,
                top
            );

            if top <= height_end {
                return Ok(())
            }

            height = height_end;
        }
    }
}