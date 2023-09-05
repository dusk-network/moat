// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::wallet_accessor::Password::{Pwd, PwdHash};
use crate::BlockchainAccessConfig;
use dusk_bls12_381::BlsScalar;
use dusk_wallet::gas::Gas;
use dusk_wallet::{SecureWalletFile, Wallet, WalletPath};
use dusk_wallet_core::MAX_CALL_SIZE;
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;
use sha2::{Digest, Sha256};
use tracing::info;

#[derive(Debug, Clone)]
pub enum Password {
    Pwd(String),
    PwdHash(String),
}

#[derive(Debug)]
pub struct WalletAccessor {
    pub path: WalletPath,
    pub pwd: Password,
    pub pwd_bytes: Vec<u8>,
}

impl SecureWalletFile for WalletAccessor {
    fn path(&self) -> &WalletPath {
        &self.path
    }

    fn pwd(&self) -> &[u8] {
        self.pwd_bytes.as_slice()
    }
}

impl WalletAccessor {
    pub fn new(path: WalletPath, pwd: Password) -> Self {
        let slf = Self {
            path,
            pwd: pwd.clone(),
            pwd_bytes: {
                match &pwd {
                    Pwd(s) => {
                        let mut hasher = Sha256::new();
                        hasher.update(s.as_bytes());
                        hasher.finalize().to_vec()
                    }
                    PwdHash(h) => {
                        hex::decode(h.as_str()).unwrap_or([0u8; 32].to_vec())
                    } // todo - how do we react to invalid hex of the hash
                }
            },
        };
        println!("pwd_bytes={:?}", hex::encode(slf.pwd_bytes.clone()));
        slf
    }

    pub async fn send<C>(
        &self,
        data: C,
        contract_id: ModuleId,
        call_name: String,
        cfg: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, dusk_wallet::Error>
    where
        C: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
    {
        let wallet_accessor =
            WalletAccessor::new(self.path.clone(), self.pwd.clone());
        let mut wallet = Wallet::from_file(wallet_accessor)?;
        wallet
            .connect_with_status(
                cfg.rusk_address.clone(),
                cfg.prover_address.clone(),
                |s| {
                    info!(target: "wallet", "{s}",);
                },
                // true,
            )
            .await?;

        assert!(wallet.is_online(), "Wallet should be online");

        info!("Sending request");
        println!("Sending request");

        let sender = wallet.default_address();
        // let rcvr = wallet.addresses().get(1).expect("address exists");
        let mut gas = Gas::new(gas_limit);
        gas.set_price(gas_price);

        let tx = wallet
            .execute(sender, contract_id, call_name, data, gas)
            .await?;
        // let tx: Transaction = wallet
        //     .transfer(sender, rcvr, Dusk::from(8), gas)
        //     .await?;
        let tx_id = rusk_abi::hash::Hasher::digest(tx.to_hash_input_bytes());
        info!("TX_ID={:x}", tx_id);
        println!("TX_ID={:x}", tx_id);
        Ok(tx_id)
    }
}
