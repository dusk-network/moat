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

    /// Directory to store user data [default: `$HOME/.dusk/rusk-wallet`]
    #[clap(short, long)]
    pub profile: PathBuf,

    /// Set the password for wallet's creation
    #[clap(long, env = "RUSK_WALLET_PWD")]
    pub password: String,

    /// Use current timestamp instead of the one specified in the input file
    #[clap(long)]
    pub now: bool,
}
