// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::MAX_CALL_SIZE;
use dusk_bls12_381::BlsScalar;
use dusk_wallet::WalletPath;
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;
use wallet_accessor::{BlockchainAccessConfig, Password, WalletAccessor};

pub struct PayloadSender;

impl PayloadSender {
    /// Sends payload to a given method
    #[allow(clippy::too_many_arguments)]
    pub async fn send_to_contract_method<P, M>(
        payload: P,
        cfg: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        password: &Password,
        gas_limit: u64,
        gas_price: u64,
        contract_id: ModuleId,
        method: M,
    ) -> Result<BlsScalar, Error>
    where
        P: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
        M: AsRef<str>,
    {
        let wallet_accessor =
            WalletAccessor::new(wallet_path.clone(), password.clone());
        let tx_id = wallet_accessor
            .execute_contract_method(
                payload,
                contract_id,
                method.as_ref().to_string(),
                cfg,
                gas_limit,
                gas_price,
            )
            .await?;
        Ok(tx_id)
    }
}
