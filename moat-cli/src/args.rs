// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Path of the JSON file to be processed
    pub json_path: PathBuf,

    /// Wallet directory [default: `$HOME/.dusk/rusk-wallet`]
    #[clap(short, long)]
    pub wallet_path: PathBuf,

    /// Config directory
    #[clap(short, long)]
    pub config_path: PathBuf,

    /// Password for the wallet
    #[clap(long, env = "RUSK_WALLET_PWD")]
    pub password: String,

    /// Gas limit [default: `500000000`]
    #[clap(short, long, default_value_t = 500000000)]
    pub gas_limit: u64,

    /// Gas price [default: `1`]
    #[clap(short, long, default_value_t = 1)]
    pub gas_price: u64,
}
