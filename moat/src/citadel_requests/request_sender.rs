// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::api::MoatContext;
use crate::{
    Error, PayloadSender, LICENSE_CONTRACT_ID, REQUEST_LICENSE_METHOD_NAME,
};
use dusk_bls12_381::BlsScalar;
use zk_citadel::license::Request;

pub struct RequestSender;

impl RequestSender {
    /// Sends (submits) request into the blockchain.
    pub async fn send_request(
        request: Request,
        moat_context: &MoatContext,
    ) -> Result<BlsScalar, Error> {
        let tx_id = PayloadSender::execute_contract_method(
            request,
            moat_context,
            LICENSE_CONTRACT_ID,
            REQUEST_LICENSE_METHOD_NAME,
        )
        .await?;
        Ok(tx_id)
    }
}
