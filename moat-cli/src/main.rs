// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod args;

use crate::args::Args;

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use dusk_wallet::WalletPath;
use tracing::Level;
use wallet_caller_lib::WalletAccessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr);
    tracing::subscriber::set_global_default(subscriber.finish())?;

    let cli = Args::parse();

    let ts_override = cli.now.then(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time should go forward")
            .as_millis() as u64
    });

    let data = json_file(cli.json_path, ts_override)?;
    WalletPath::set_cache_dir(&cli.profile)?;
    let wallet_path =
        WalletPath::from(cli.profile.as_path().join("wallet.dat"));
    let config_path = cli.profile.as_path().join("gov_config.toml");

    let wallet_accessor = WalletAccessor {
        pwd: cli.password,
        path: wallet_path,
    };

    // let contract = Governance::new(wallet, config_path)?;
    //
    // contract.send_data(data).await?;

    Ok(())
}
