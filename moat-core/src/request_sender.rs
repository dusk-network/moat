// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use dusk_jubjub::BlsScalar;
use dusk_wallet::WalletPath;
use rusk_abi::ModuleId;
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};
use zk_citadel::license::Request;

pub struct RequestSender;

const LICENSE_CONTRACT_ID: ModuleId = {
    let mut bytes = [0u8; 32];
    bytes[0] = 0x01; // 0xf8; todo: - temporarily we make it the TRANSFER contract
    ModuleId::from_bytes(bytes)
};

const METHOD_NAME: &str = "root"; // todo: - temporarily we make it root, it should be License contract's noop

impl RequestSender {
    /// main orchestrating function sending requests to the license contract
    /// it drives wallet accessor instantiation
    /// accepts Request as argument
    pub async fn send(
        request: Request,
        cfg: &BlockchainAccessConfig,
        wallet_path: WalletPath,
        password: String,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, Error> {
        let wallet_accessor = WalletAccessor {
            path: wallet_path,
            pwd: password,
        };
        let tx_id = wallet_accessor
            .send(
                request,
                LICENSE_CONTRACT_ID,
                METHOD_NAME.to_string(),
                cfg,
                gas_limit,
                gas_price,
            )
            .await?;
        Ok(tx_id)
    }
}
