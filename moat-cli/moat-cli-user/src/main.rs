// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(stmt_expr_attributes)]

mod args;
mod command;
mod interactor;
mod menu;
mod prompt;
mod run_result;

use crate::args::Args;
use crate::command::Command;
use crate::menu::Menu;

use clap::Parser;

use crate::interactor::Interactor;
use moat_cli_common::Error;
use rand::SeedableRng;

use zk_citadel_moat::api::MoatContext;

#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Error> {
    let cli = Args::parse();

    let config_path = cli.config_path;
    let wallet_path = cli.wallet_path;
    let password = cli.wallet_pass;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;

    let moat_context = MoatContext::create(
        config_path,
        wallet_path,
        password,
        gas_limit,
        gas_price,
    )
    .await?;

    let mut interactor = Interactor { moat_context };

    interactor.run_loop().await?;

    Ok(())
}
