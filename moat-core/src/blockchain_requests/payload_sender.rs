// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::{
    ISSUE_LICENSE_METHOD_NAME, LICENSE_CONTRACT_ID, MAX_CALL_SIZE,
    NOOP_METHOD_NAME, USE_LICENSE_METHOD_NAME,
};
use dusk_jubjub::BlsScalar;
use dusk_wallet::WalletPath;
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;
use wallet_accessor::{BlockchainAccessConfig, Password, WalletAccessor};

pub struct PayloadSender;

impl PayloadSender {
    /// Sends a given payload to the noop method
    pub async fn send_noop<P>(
        payload: P,
        cfg: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        password: &Password,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, Error>
    where
        P: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
    {
        Self::send_to_contract_method(
            payload,
            cfg,
            wallet_path,
            password,
            gas_limit,
            gas_price,
            LICENSE_CONTRACT_ID,
            NOOP_METHOD_NAME,
        )
        .await
    }

    /// Sends a given payload to the issue license method
    pub async fn send_issue_license<P>(
        payload: P,
        cfg: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        password: &Password,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, Error>
    where
        P: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
    {
        Self::send_to_contract_method(
            payload,
            cfg,
            wallet_path,
            password,
            gas_limit,
            gas_price,
            LICENSE_CONTRACT_ID,
            ISSUE_LICENSE_METHOD_NAME,
        )
        .await
    }

    /// Sends a given payload to the use license method
    pub async fn send_use_license<P>(
        payload: P,
        cfg: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        password: &Password,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, Error>
    where
        P: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
    {
        Self::send_to_contract_method(
            payload,
            cfg,
            wallet_path,
            password,
            gas_limit,
            gas_price,
            LICENSE_CONTRACT_ID,
            USE_LICENSE_METHOD_NAME,
        )
        .await
    }

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
            .send(
                // (payload, 1u64, BlsScalar::one()),
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
