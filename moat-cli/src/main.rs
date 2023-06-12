// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(stmt_expr_attributes)]
mod args;

use crate::args::Args;

use std::error::Error;
use toml_base_config::BaseConfig;

use clap::Parser;
use dusk_wallet::WalletPath;
use moat_core::{RequestJson, RequestSender};
use tracing::Level;
use wallet_accessor::BlockchainAccessConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr);
    tracing::subscriber::set_global_default(subscriber.finish())?;

    let cli = Args::parse();

    let json_path = cli.json_path.as_path();
    let config_path = cli.config_path.as_path();
    let wallet_path = cli.wallet_path.as_path();
    let password = cli.password;

    let request_json = RequestJson::from_file(json_path)?;
    let request = request_json.to_request();

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    RequestSender::send(
        request,
        &blockchain_access_config,
        wallet_path,
        password,
    )
    .await?;

    #[rustfmt::skip]
    // cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --password hyundai23! ./moat-cli/request.json

    Ok(())
}
