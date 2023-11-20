// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(stmt_expr_attributes)]

mod args;
mod command;
mod config;
mod interactor;
mod menu;
mod prompt;
mod run_result;

use crate::args::Args;
use crate::command::Command;
use crate::menu::Menu;
use std::fs;

use clap::Parser;

use crate::config::LPCliConfig;
use crate::interactor::Interactor;
use dusk_wallet::{Wallet, WalletPath};
use moat_cli_common::Error;
use rand::SeedableRng;
use toml_base_config::BaseConfig;
use wallet_accessor::Password::{Pwd, PwdHash};
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Args::parse();

    let config_path = cli.config_path.as_path();
    let wallet_path = cli.wallet_path.as_path();
    let password = cli.wallet_pass;
    let pwd_hash = cli.pwd_hash;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;

    let _ = fs::metadata(config_path).map_err(|_| {
        Error::NotFound(config_path.to_string_lossy().into_owned().into())
    })?;
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

    let wallet_accessor =
        WalletAccessor::create(wallet_path.clone(), psw.clone()).unwrap();
    let wallet = Wallet::from_file(wallet_accessor).unwrap();

    let (_psk, ssk) = wallet.spending_keys(wallet.default_address()).unwrap();

    let mut interactor = Interactor {
        wallet_path,
        psw,
        blockchain_access_config,
        ssk,
        gas_limit,
        gas_price,
    };

    interactor.run_loop().await?;

    #[rustfmt::skip]
        // old wallet.dat file format:
        // cargo r --release --bin moat-cli-lp -- --wallet-path ~/.dusk/rusk-wallet --pwd-hash 7f2611ba158b6dcea4a69c229c303358c5e04493abeadee106a4bfa464d55787
        // new wallet.dat file format:
        // cargo r --release --bin moat-cli-lp -- --wallet-path ~/.dusk/rusk-wallet --pwd-hash 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8

        Ok(())
}
