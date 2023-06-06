// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_wallet::{SecureWalletFile, WalletPath};
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
