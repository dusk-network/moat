// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Command line utility for submitting requests to the Dusk blockchain.
//!
//! In order to use the moat-cli utility you need to install Dusk wallet first.
//! Typically, your Dusk wallet is installed in ~/.dusk/rusk-wallet.
//! Path to your wallet's directory needs to be provided as `wallet-path`
//! argument when using moat-cli.
//! Before usage you need to make sure that the default address of your wallet
//! holds some Dusk funds, otherwise the utility won't be able to submit your
//! request, as it needs funds for gas in order to do so.
//! The CLI will also need password to your wallet, as well as a path to
//! a configuration file containing blockchain urls. Example configuration file
//! is provided in the moat-cli project main directory, under the name
//! `config.toml`. The last thing you will need is an actual request. You will
//! be able to provide it in a form of a json file. An example request file is
//! provided in the moat-cli project's main directory as `request.json`.
//!
//! Note that your wallet cannot be active when running this utility.
//!
//! To sum up - you'll need a wallet with some funds in its default address,
//! a path to its location, wallet's password, blockchain access configuration
//! file `config.toml`, and a request file.
//!
//! In addition, you may also want to provide gas limit and gas price, also via
//! command line parameters.
//!
//! To sum up, the exact list of command line parameters is as follows:
//!
//! - `wallet_path` - a path to wallet's location, e.g.: `--wallet_path
//!   ~/.dusk/rusk-wallet`
//! - `config_path` - a path to configuratin file, e.g.: `--config_path
//!   ./moat-cli/config.toml`
//! - `password` - wallet's password in the clear, e.g: `--password mypass2!`
//! - `psw_hash` - wallet's password's blake3 hash, e.g: `--psw_hash
//!   7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d5aabb`
//! - `gas_limit` - gas limit, e.g.: `--gas_limit 500000000`
//! - `gas_price` - gas price, e.g.: `--gas_price 1`
//! - a full path (with a name) of the request file, e.g.:
//!   `./moat-cli/request.json`
//!
//! Example full command line invocation of `moat-cli` may look as follows:
//!
//! `cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet
//! --config-path ./moat-cli/config.toml
//! --psw_hash 7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d5aabb
//! ./moat-cli/request.json`
//!
//! Note that when psw_hash argument is provided, password argument may be
//! omitted or if given, it will be ignored.

#![feature(stmt_expr_attributes)]
mod args;

use crate::args::Args;

use clap::Parser;
use dusk_wallet::WalletPath;
use moat_core::{JsonLoader, PayloadSender, RequestCreator, RequestJson};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::error::Error;
use toml_base_config::BaseConfig;
use tracing::Level;
use wallet_accessor::{
    BlockchainAccessConfig, Password::Pwd, Password::PwdHash,
};

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
    let pwd_hash = cli.pwd_hash;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;

    let request_json: RequestJson = RequestJson::from_file(json_path)?;
    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;
    let psw = if pwd_hash.is_empty() {
        Pwd(password)
    } else {
        PwdHash(pwd_hash)
    };

    PayloadSender::send(
        request,
        &blockchain_access_config,
        wallet_path,
        psw,
        gas_limit,
        gas_price,
    )
    .await?;

    #[rustfmt::skip]
    // cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --password password ./moat-cli/request.json
    // cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --pwd-hash 7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787 ./moat-cli/request.json

    Ok(())
}
