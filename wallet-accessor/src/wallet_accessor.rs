// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::BlockchainAccessConfig;
use dusk_bls12_381::BlsScalar;
use dusk_wallet::gas::Gas;
use dusk_wallet::{SecureWalletFile, TransportTCP, Wallet, WalletPath};
use dusk_wallet_core::{Transaction, MAX_CALL_SIZE};
use rkyv::ser::serializers::AllocSerializer;
use rusk_abi::ModuleId;
use tracing::info;

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

impl WalletAccessor {
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
        let wallet_accessor = WalletAccessor {
            path: self.path.clone(),
            pwd: self.pwd.clone(),
        };
        let mut wallet = Wallet::from_file(wallet_accessor)?;
        let transport_tcp = TransportTCP::new(
            cfg.rusk_address.clone(),
            cfg.prover_address.clone(),
        );

        wallet
            .connect_with_status(transport_tcp, |s| {
                info!(target: "wallet", "{s}",);
            })
            .await?;

        assert!(wallet.is_online(), "Wallet should be online");

        // todo: we might add gql here to be able to obtain
        // confirmation that transaction has been successfully submitted
        // let gql = GraphQL::new(cfg.graphql_address.clone(), |s| {
        //     tracing::info!(target: "graphql", "{s}",);
        // });

        info!("Sending request");

        let sender = wallet.default_address();
        // let rcvr = wallet.addresses().get(1).expect("address exists");
        let mut gas = Gas::new(gas_limit);
        gas.set_price(gas_price);

        let tx: Transaction = wallet
            .execute(sender, contract_id, call_name, data, gas)
            .await?;
        // let tx: Transaction = wallet
        //     .transfer(sender, rcvr, Dusk::from(8), gas)
        //     .await?;
        let tx_id = rusk_abi::hash(tx.to_hash_input_bytes());
        info!("TX_ID={:x}", tx_id);
        // gql.wait_for(&tx_id).await?;
        Ok(tx_id)
    }
}
