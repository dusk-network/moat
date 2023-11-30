// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::wallet_accessor::{BlockchainAccessConfig, Password};
use crate::{
    Error, PayloadSender, LICENSE_CONTRACT_ID, REQUEST_LICENSE_METHOD_NAME,
};
use dusk_bls12_381::BlsScalar;
use dusk_wallet::WalletPath;
use zk_citadel::license::Request;

pub struct RequestSender;

impl RequestSender {
    pub async fn send_request(
        request: Request,
        config: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        password: &Password,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, Error> {
        let tx_id = PayloadSender::execute_contract_method(
            request,
            config,
            wallet_path,
            password,
            gas_limit,
            gas_price,
            LICENSE_CONTRACT_ID,
            REQUEST_LICENSE_METHOD_NAME,
        )
        .await?;
        Ok(tx_id)
    }
}
