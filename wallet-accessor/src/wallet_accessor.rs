// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::gas::Gas;
use dusk_wallet::{SecureWalletFile, Wallet, WalletPath};
use dusk_wallet_core::{MAX_CALL_SIZE, Transaction};
use rusk_abi::ModuleId;
use dusk_bls12_381::BlsScalar;
use rkyv::ser::serializers::AllocSerializer;
use serde::{Deserialize, Serialize};
use toml_base_config::BaseConfig;

#[derive(Default, Deserialize, Serialize)]
pub struct WalletAccessorConfig {
    pub rusk_address: String,
    pub prover_address: String,
    pub graphql_address: String,
    pub gas_limit: u64,
    pub gas_price: Option<u64>,
}

#[derive(Debug)]
pub struct WalletAccessor {
    pub path: WalletPath,
    pub pwd: String,
}

impl SecureWalletFile for WalletAccessor {
    fn path(&self) -> &WalletPath {
        &self.path
    }

    fn pwd(&self) -> blake3::Hash {
        blake3::hash(self.pwd.as_bytes())
    }
}

impl BaseConfig for WalletAccessorConfig {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}

impl WalletAccessor {
    pub async fn send<C>(
        &self,
        data: C,
        contract_id: ModuleId,
        gas_limit: u64,
        gas_price: Option<u64>,
    ) -> Result<Vec<u8>, dusk_wallet::Error>
    where
        C: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
    {
        let wallet_accessor = WalletAccessor { path: self.path.clone(), pwd: self.pwd.clone()};
        let mut wallet = Wallet::from_file(wallet_accessor)?;
        let sender = wallet.default_address();
        let mut gas = Gas::new(gas_limit);
        gas.set_price(gas_price);
        let tx: Transaction = wallet.execute(sender, contract_id, "TODO".to_string(), data, gas).await?;
        Ok(tx.to_hash_input_bytes())
    }
}
