// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Wallet directory [default: `$HOME/.dusk/rusk-wallet`]
    #[clap(short, long, default_value = concat!(env!("HOME"), "/.dusk/rusk-wallet/wallet.dat"))]
    pub wallet_path: String,

    /// Blockchain access config directory
    #[clap(short, long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../config.toml"))]
    pub config_path: String,

    /// Password for the wallet
    #[clap(long, default_value_t = String::from(""), env = "RUSK_WALLET_PWD")]
    pub wallet_pass: String,

    /// Hash of the password for the wallet [default: ``]
    #[clap(short, long, default_value_t = String::from(""))]
    pub pwd_hash: String,

    /// Gas limit [default: `500000000`]
    #[clap(long, default_value_t = 500000000)]
    pub gas_limit: u64,

    /// Gas price [default: `1`]
    #[clap(long, default_value_t = 1)]
    pub gas_price: u64,
}
