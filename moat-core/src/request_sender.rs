// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::request::Request;
use dusk_wallet::WalletPath;
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};
use rusk_abi::ModuleId;


pub struct RequestSender;

const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0xf8;
    ModuleId::from_bytes(bytes)
};

impl RequestSender {
    /// main orchestrating function sending requests to the license contract
    /// it drives wallet accessor instantiation
    /// accepts Request as argument
    pub async fn send(
        request: Request,
        cfg: &BlockchainAccessConfig,
        wallet_path: WalletPath,
        password: String,
    ) -> Result<(), Error> {
        let wallet_accessor = WalletAccessor {
            path: wallet_path,
            pwd: password,
        };
        wallet_accessor.send(request, LICENSE_CONTRACT_ID, "noop".to_string(), cfg).await?;
        Ok(())
    }
}
