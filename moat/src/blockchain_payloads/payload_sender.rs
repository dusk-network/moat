// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::api::MoatContext;
use crate::error::Error;
use crate::wallet_accessor::accessor;

use crate::MAX_CALL_SIZE;

use dusk_bls12_381::BlsScalar;
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;

pub struct PayloadSender;

impl PayloadSender {
    /// Sends payload to a given method
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_contract_method<P, M>(
        payload: P,
        moat_context: &MoatContext,
        contract_id: ModuleId,
        method: M,
    ) -> Result<BlsScalar, Error>
    where
        P: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
        M: AsRef<str>,
    {
        let tx_id = accessor::execute_contract_method(
            moat_context,
            payload,
            contract_id,
            method.as_ref().to_string(),
        )
        .await?;
        Ok(tx_id)
    }
}
