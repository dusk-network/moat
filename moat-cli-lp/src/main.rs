// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(stmt_expr_attributes)]

mod args;
mod command;
mod config;
mod error;
mod interactor;
mod menu;
mod prompt;
mod run_result;

use crate::args::Args;
use crate::command::Command;
use crate::menu::Menu;

use clap::Parser;

use crate::config::LPCliConfig;
use crate::error::CliError;
use crate::interactor::Interactor;
use dusk_wallet::WalletPath;
use rand::SeedableRng;
use toml_base_config::BaseConfig;
use wallet_accessor::BlockchainAccessConfig;
use wallet_accessor::Password::{Pwd, PwdHash};

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Args::parse();

    let config_path = cli.config_path.as_path();
    let wallet_path = cli.wallet_path.as_path();
    let password = cli.password;
    let pwd_hash = cli.pwd_hash;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;

    let config = LPCliConfig::load_path(config_path)?;
    let blockchain_access_config = BlockchainAccessConfig {
        rusk_address: config.rusk_address.clone(),
        prover_address: config.prover_address.clone(),
    };

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let psw = if pwd_hash.is_empty() {
        Pwd(password)
    } else {
        PwdHash(pwd_hash)
    };

    let mut interactor = Interactor {
        wallet_path,
        psw,
        blockchain_access_config,
        config,
        gas_limit,
        gas_price,
    };

    interactor.run_loop().await?;

    #[rustfmt::skip]
        // old wallet.dat file format:
        // cargo r --release --bin moat-cli-lp -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli-lp/config.toml --pwd-hash 7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787
        // new wallet.dat file format:
        // cargo r --release --bin moat-cli-lp -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli-lp/config.toml --pwd-hash 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8

        Ok(())
}
