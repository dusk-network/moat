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

use clap::Parser;

use crate::config::SPCliConfig;
use crate::interactor::Interactor;
use dusk_wallet::{Wallet, WalletPath};
use moat_cli_common::Error;
use toml_base_config::BaseConfig;
use zk_citadel_moat::wallet_accessor::Password::{Pwd, PwdHash};
use zk_citadel_moat::wallet_accessor::{
    BlockchainAccessConfig, WalletAccessor,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Args::parse();

    let config_path = cli.config_path.as_path();
    let wallet_path = cli.wallet_path.as_path();
    let password = cli.wallet_pass;
    let pwd_hash = cli.pwd_hash;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;

    let config = SPCliConfig::load_path(config_path)?;
    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let psw = if pwd_hash.is_empty() {
        Pwd(password)
    } else {
        PwdHash(pwd_hash)
    };

    let wallet_accessor =
        WalletAccessor::create(wallet_path.clone(), psw.clone())?;
    let wallet = Wallet::from_file(wallet_accessor)?;

    let (psk_sp, _ssk_sp) = wallet.spending_keys(wallet.default_address())?;

    let mut interactor = Interactor {
        wallet_path,
        psw,
        blockchain_access_config,
        config,
        gas_limit,
        gas_price,
        psk_sp,
    };

    interactor.run_loop().await?;

    Ok(())
}
