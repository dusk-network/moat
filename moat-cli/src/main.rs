// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(stmt_expr_attributes)]
mod args;

use crate::args::Args;

use clap::Parser;
use dusk_wallet::WalletPath;
use moat_core::{RequestCreator, RequestJson, RequestSender};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::error::Error;
use toml_base_config::BaseConfig;
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
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;

    let request_json = RequestJson::from_file(json_path)?;
    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    RequestSender::send(
        request,
        &blockchain_access_config,
        wallet_path,
        password,
        gas_limit,
        gas_price,
    )
    .await?;

    #[rustfmt::skip]
    // cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --password hyundai23! ./moat-cli/request.json

    Ok(())
}
